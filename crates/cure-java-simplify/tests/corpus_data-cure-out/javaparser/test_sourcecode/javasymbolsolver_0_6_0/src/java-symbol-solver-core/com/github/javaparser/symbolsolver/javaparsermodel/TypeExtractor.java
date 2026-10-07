package com.github.javaparser.symbolsolver.javaparsermodel;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.NodeList;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.*;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.ExpressionStmt;
import com.github.javaparser.ast.stmt.ReturnStmt;
import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.ast.type.UnknownType;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.logic.FunctionalInterfaceLogic;
import com.github.javaparser.symbolsolver.logic.InferenceContext;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.*;
import com.github.javaparser.symbolsolver.reflectionmodel.MyObjectProvider;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionClassDeclaration;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import com.github.javaparser.symbolsolver.resolution.typesolvers.ReflectionTypeSolver;
import com.google.common.collect.ImmutableList;
import java.util.List;
import java.util.Optional;
import java.util.logging.ConsoleHandler;
import java.util.logging.Level;
import java.util.logging.Logger;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class TypeExtractor extends DefaultVisitorAdapter {
    private static Logger logger = Logger.getLogger(TypeExtractor.class.getCanonicalName());
    static {
        logger.setLevel(Level.INFO);
        ConsoleHandler consoleHandler = new ConsoleHandler();
        consoleHandler.setLevel(Level.INFO);
        logger.addHandler(consoleHandler);
    }
    private TypeSolver typeSolver;
    private JavaParserFacade facade;
    public TypeExtractor(TypeSolver typeSolver, JavaParserFacade facade) {
        this.typeSolver = typeSolver;
        this.facade = facade;
    }
    @Override
    public Type visit(VariableDeclarator node, Boolean solveLambdas) {
        if (getParentNode(node) instanceof FieldDeclaration) {
            return facade.convertToUsageVariableType(node);
        } else if (getParentNode(node) instanceof VariableDeclarationExpr) {
            return facade.convertToUsageVariableType(node);
        } else {
            throw new UnsupportedOperationException(getParentNode(node).getClass().getCanonicalName());
        }
    }
    @Override
    public Type visit(Parameter node, Boolean solveLambdas) {
        if (node.getType() instanceof UnknownType) {
            throw new IllegalStateException("Parameter has unknown type: " + node);
        }
        return facade.convertToUsage(node.getType(), node);
    }
    @Override
    public Type visit(ArrayAccessExpr node, Boolean solveLambdas) {
        Type arrayUsageType = node.getName().accept(this, solveLambdas);
        return arrayUsageType.isArray() ? ((ArrayType) arrayUsageType).getComponentType() : arrayUsageType;
    }
    @Override
    public Type visit(ArrayCreationExpr node, Boolean solveLambdas) {
        Type res = facade.convertToUsage(node.getElementType(), JavaParserFactory.getContext(node, typeSolver));
        for (int i = 0; i < node.getLevels().size(); i++) {
            res = new ArrayType(res);
        }
        return res;
    }
    @Override
    public Type visit(ArrayInitializerExpr node, Boolean solveLambdas) {
        throw new UnsupportedOperationException(node.getClass().getCanonicalName());
    }
    @Override
    public Type visit(AssignExpr node, Boolean solveLambdas) {
        return node.getTarget().accept(this, solveLambdas);
    }
    @Override
    public Type visit(BinaryExpr node, Boolean solveLambdas) {
        switch (node.getOperator()) {
            case PLUS:
            case MINUS:
            case DIVIDE:
            case MULTIPLY:
                return facade.getBinaryTypeConcrete(node.getLeft(), node.getRight(), solveLambdas);
            case LESS_EQUALS:
            case LESS:
            case GREATER:
            case GREATER_EQUALS:
            case EQUALS:
            case NOT_EQUALS:
            case OR:
            case AND:
                return PrimitiveType.BOOLEAN;
            case BINARY_AND:
            case BINARY_OR:
            case SIGNED_RIGHT_SHIFT:
            case UNSIGNED_RIGHT_SHIFT:
            case LEFT_SHIFT:
            case REMAINDER:
            case XOR:
                return node.getLeft().accept(this, solveLambdas);
            default:
                throw new UnsupportedOperationException("FOO " + node.getOperator().name());
        }
    }
    @Override
    public Type visit(CastExpr node, Boolean solveLambdas) {
        return facade.convertToUsage(node.getType(), JavaParserFactory.getContext(node, typeSolver));
    }
    @Override
    public Type visit(ClassExpr node, Boolean solveLambdas) {
        com.github.javaparser.ast.type.Type astType = node.getType();
        Type jssType = facade.convertToUsage(astType, node.getType());
        return new ReferenceTypeImpl(new ReflectionClassDeclaration(Class.class, typeSolver), ImmutableList.of(jssType), typeSolver);
    }
    @Override
    public Type visit(ConditionalExpr node, Boolean solveLambdas) {
        return node.getThenExpr().accept(this, solveLambdas);
    }
    @Override
    public Type visit(EnclosedExpr node, Boolean solveLambdas) {
        return node.getInner().accept(this, solveLambdas);
    }
    private Type solveDotExpressionType(ReferenceTypeDeclaration parentType, FieldAccessExpr node) {
        if (parentType.hasField(node.getName().getId())) {
            return parentType.getField(node.getName().getId()).getType();
        } else if (parentType.hasInternalType(node.getName().getId())) {
            return new ReferenceTypeImpl(parentType.getInternalType(node.getName().getId()), typeSolver);
        } else {
            throw new UnsolvedSymbolException(node.getName().getId());
        }
    }
    @Override
    public Type visit(FieldAccessExpr node, Boolean solveLambdas) {
        if (node.getScope() instanceof NameExpr || node.getScope() instanceof FieldAccessExpr) {
            Expression staticValue = node.getScope();
            SymbolReference<TypeDeclaration> typeAccessedStatically = JavaParserFactory.getContext(node, typeSolver).solveType(staticValue.toString(), typeSolver);
            if (typeAccessedStatically.isSolved()) {
                return solveDotExpressionType(typeAccessedStatically.getCorrespondingDeclaration().asReferenceType(), node);
            }
        } else if (node.getScope() instanceof ThisExpr) {
            SymbolReference<TypeDeclaration> solve = facade.solve((ThisExpr) node.getScope());
            if (solve.isSolved()) {
                TypeDeclaration correspondingDeclaration = solve.getCorrespondingDeclaration();
                if (correspondingDeclaration instanceof ReferenceTypeDeclaration) {
                    return solveDotExpressionType(correspondingDeclaration.asReferenceType(), node);
                }
            }
        } else if (node.getScope().toString().indexOf('.') > 0) {
            SymbolReference<ReferenceTypeDeclaration> sr = typeSolver.tryToSolveType(node.getScope().toString());
            if (sr.isSolved()) {
                return solveDotExpressionType(sr.getCorrespondingDeclaration(), node);
            }
        }
        Optional<Value> value = null;
        try {
            value = new SymbolSolver(typeSolver).solveSymbolAsValue(node.getField().getId(), node);
        } catch (UnsolvedSymbolException use) {
            SymbolReference<ReferenceTypeDeclaration> sref = typeSolver.tryToSolveType(node.toString());
            if (sref.isSolved()) {
                return new ReferenceTypeImpl(sref.getCorrespondingDeclaration(), typeSolver);
            }
        }
        if (value != null && value.isPresent()) {
            return value.get().getType();
        } else {
            throw new UnsolvedSymbolException(node.getField().getId());
        }
    }
    @Override
    public Type visit(InstanceOfExpr node, Boolean solveLambdas) {
        return PrimitiveType.BOOLEAN;
    }
    @Override
    public Type visit(StringLiteralExpr node, Boolean solveLambdas) {
        return new ReferenceTypeImpl(new ReflectionTypeSolver().solveType("java.lang.String"), typeSolver);
    }
    @Override
    public Type visit(IntegerLiteralExpr node, Boolean solveLambdas) {
        return PrimitiveType.INT;
    }
    @Override
    public Type visit(LongLiteralExpr node, Boolean solveLambdas) {
        return PrimitiveType.LONG;
    }
    @Override
    public Type visit(CharLiteralExpr node, Boolean solveLambdas) {
        return PrimitiveType.CHAR;
    }
    @Override
    public Type visit(DoubleLiteralExpr node, Boolean solveLambdas) {
        return node.getValue().toLowerCase().endsWith("f") ? PrimitiveType.FLOAT : PrimitiveType.DOUBLE;
    }
    @Override
    public Type visit(BooleanLiteralExpr node, Boolean solveLambdas) {
        return PrimitiveType.BOOLEAN;
    }
    @Override
    public Type visit(NullLiteralExpr node, Boolean solveLambdas) {
        return NullType.INSTANCE;
    }
    @Override
    public Type visit(MethodCallExpr node, Boolean solveLambdas) {
        logger.finest("getType on method call " + node);
        MethodUsage ref = facade.solveMethodAsUsage(node);
        logger.finest("getType on method call " + node + " resolved to " + ref);
        logger.finest("getType on method call " + node + " return type is " + ref.returnType());
        return ref.returnType();
    }
    @Override
    public Type visit(NameExpr node, Boolean solveLambdas) {
        logger.finest("getType on name expr " + node);
        Optional<Value> value = new SymbolSolver(typeSolver).solveSymbolAsValue(node.getName().getId(), node);
        if (!value.isPresent()) {
            throw new UnsolvedSymbolException("Solving " + node, node.getName().getId());
        } else {
            return value.get().getType();
        }
    }
    @Override
    public Type visit(ObjectCreationExpr node, Boolean solveLambdas) {
        return facade.convertToUsage(node.getType(), node);
    }
    @Override
    public Type visit(ThisExpr node, Boolean solveLambdas) {
        if (node.getClassExpr().isPresent()) {
            String className = node.getClassExpr().get().toString();
            SymbolReference<ReferenceTypeDeclaration> clazz = typeSolver.tryToSolveType(className);
            if (clazz.isSolved()) {
                return new ReferenceTypeImpl(clazz.getCorrespondingDeclaration(), typeSolver);
            }
            Optional<CompilationUnit> cu = node.getAncestorOfType(CompilationUnit.class);
            if (cu.isPresent()) {
                Optional<ClassOrInterfaceDeclaration> classByName = cu.get().getClassByName(className);
                if (classByName.isPresent()) {
                    return new ReferenceTypeImpl(facade.getTypeDeclaration(classByName.get()), typeSolver);
                }
            }
        }
        return new ReferenceTypeImpl(facade.getTypeDeclaration(facade.findContainingTypeDecl(node)), typeSolver);
    }
    @Override
    public Type visit(SuperExpr node, Boolean solveLambdas) {
        TypeDeclaration typeOfNode = facade.getTypeDeclaration(facade.findContainingTypeDecl(node));
        if (typeOfNode instanceof ClassDeclaration) {
            return ((ClassDeclaration) typeOfNode).getSuperClass();
        } else {
            throw new UnsupportedOperationException(node.getClass().getCanonicalName());
        }
    }
    @Override
    public Type visit(UnaryExpr node, Boolean solveLambdas) {
        switch (node.getOperator()) {
            case MINUS:
            case PLUS:
                return node.getExpression().accept(this, solveLambdas);
            case LOGICAL_COMPLEMENT:
                return PrimitiveType.BOOLEAN;
            case POSTFIX_DECREMENT:
            case PREFIX_DECREMENT:
            case POSTFIX_INCREMENT:
            case PREFIX_INCREMENT:
                return node.getExpression().accept(this, solveLambdas);
            default:
                throw new UnsupportedOperationException(node.getOperator().name());
        }
    }
    @Override
    public Type visit(VariableDeclarationExpr node, Boolean solveLambdas) {
        if (node.getVariables().size() != 1) {
            throw new UnsupportedOperationException();
        }
        return facade.convertToUsageVariableType(node.getVariables().get(0));
    }
    @Override
    public Type visit(LambdaExpr node, Boolean solveLambdas) {
        if (getParentNode(node) instanceof MethodCallExpr) {
            MethodCallExpr callExpr = (MethodCallExpr) getParentNode(node);
            int pos = JavaParserSymbolDeclaration.getParamPos(node);
            SymbolReference<MethodDeclaration> refMethod = facade.solve(callExpr);
            if (!refMethod.isSolved()) {
                throw new UnsolvedSymbolException(getParentNode(node).toString(), callExpr.getName().getId());
            }
            logger.finest("getType on lambda expr " + refMethod.getCorrespondingDeclaration().getName());
            if (solveLambdas) {
                Type result = refMethod.getCorrespondingDeclaration().getParam(pos).getType();
                if (callExpr.getScope().isPresent()) {
                    Expression scope = callExpr.getScope().get();
                    boolean staticCall = false;
                    if (scope instanceof NameExpr) {
                        NameExpr nameExpr = (NameExpr) scope;
                        try {
                            SymbolReference<TypeDeclaration> type = JavaParserFactory.getContext(nameExpr, typeSolver).solveType(nameExpr.getName().getId(), typeSolver);
                            if (type.isSolved()) {
                                staticCall = true;
                            }
                        } catch (Exception e) {}
                    }
                    if (!staticCall) {
                        Type scopeType = facade.getType(scope);
                        if (scopeType.isReferenceType()) {
                            result = scopeType.asReferenceType().useThisTypeParametersOnTheGivenType(result);
                        }
                    }
                }
                Context ctx = JavaParserFactory.getContext(node, typeSolver);
                result = facade.solveGenericTypes(result, ctx, typeSolver);
                Optional<MethodUsage> functionalMethod = FunctionalInterfaceLogic.getFunctionalMethod(result);
                if (functionalMethod.isPresent()) {
                    LambdaExpr lambdaExpr = node;
                    InferenceContext lambdaCtx = new InferenceContext(MyObjectProvider.INSTANCE);
                    InferenceContext funcInterfaceCtx = new InferenceContext(MyObjectProvider.INSTANCE);
                    Type functionalInterfaceType = ReferenceTypeImpl.undeterminedParameters(functionalMethod.get().getDeclaration().declaringType(), typeSolver);
                    lambdaCtx.addPair(result, functionalInterfaceType);
                    Type actualType;
                    if (lambdaExpr.getBody() instanceof ExpressionStmt) {
                        actualType = facade.getType(((ExpressionStmt) lambdaExpr.getBody()).getExpression());
                    } else if (lambdaExpr.getBody() instanceof BlockStmt) {
                        BlockStmt blockStmt = (BlockStmt) lambdaExpr.getBody();
                        NodeList<Statement> statements = blockStmt.getStatements();
                        List<ReturnStmt> returnStmts = blockStmt.getNodesByType(ReturnStmt.class);
                        if (returnStmts.size() > 0) {
                            actualType = returnStmts.stream().map((returnStmt) -> {
                                Optional<Expression> expression = returnStmt.getExpression();
                                return expression.isPresent() ? facade.getType(expression.get()) : VoidType.INSTANCE;
                            }).filter((x) -> x != null && !x.isVoid() && !x.isNull()).findFirst().orElse(VoidType.INSTANCE);
                        } else {
                            return VoidType.INSTANCE;
                        }
                    } else {
                        throw new UnsupportedOperationException();
                    }
                    Type formalType = functionalMethod.get().returnType();
                    funcInterfaceCtx.addPair(formalType, actualType);
                    Type functionalTypeWithReturn = funcInterfaceCtx.resolve(funcInterfaceCtx.addSingle(functionalInterfaceType));
                    if (!(formalType instanceof VoidType)) {
                        lambdaCtx.addPair(result, functionalTypeWithReturn);
                        result = lambdaCtx.resolve(lambdaCtx.addSingle(result));
                    }
                }
                return result;
            } else {
                return refMethod.getCorrespondingDeclaration().getParam(pos).getType();
            }
        } else {
            throw new UnsupportedOperationException("The type of a lambda expr depends on the position and its return value");
        }
    }
    @Override
    public Type visit(MethodReferenceExpr node, Boolean solveLambdas) {
        if (getParentNode(node) instanceof MethodCallExpr) {
            MethodCallExpr callExpr = (MethodCallExpr) getParentNode(node);
            int pos = JavaParserSymbolDeclaration.getParamPos(node);
            SymbolReference<com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration> refMethod = facade.solve(callExpr, false);
            if (!refMethod.isSolved()) {
                throw new UnsolvedSymbolException(getParentNode(node).toString(), callExpr.getName().getId());
            }
            logger.finest("getType on method reference expr " + refMethod.getCorrespondingDeclaration().getName());
            if (solveLambdas) {
                Type result = facade.solveMethodAsUsage(callExpr).getParamType(pos);
                Context ctx = JavaParserFactory.getContext(node, typeSolver);
                result = facade.solveGenericTypes(result, ctx, typeSolver);
                Optional<MethodUsage> functionalMethod = FunctionalInterfaceLogic.getFunctionalMethod(result);
                if (functionalMethod.isPresent()) {
                    if (node instanceof MethodReferenceExpr) {
                        Type actualType = facade.toMethodUsage(node).returnType();
                        Type formalType = functionalMethod.get().returnType();
                        InferenceContext inferenceContext = new InferenceContext(MyObjectProvider.INSTANCE);
                        inferenceContext.addPair(formalType, actualType);
                        result = inferenceContext.resolve(inferenceContext.addSingle(result));
                    }
                }
                return result;
            } else {
                return refMethod.getCorrespondingDeclaration().getParam(pos).getType();
            }
        } else {
            throw new UnsupportedOperationException("The type of a method reference expr depends on the position and its return value");
        }
    }
}
