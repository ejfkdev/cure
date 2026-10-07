package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.LambdaExpr;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.logic.FunctionalInterfaceLogic;
import com.github.javaparser.symbolsolver.logic.InferenceContext;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.LambdaConstraintType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.MyObjectProvider;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import javaslang.Tuple2;
import java.util.*;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class LambdaExprContext extends AbstractJavaParserContext<LambdaExpr> {
    public LambdaExprContext(LambdaExpr wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        for (Parameter parameter : wrappedNode.getParameters()) {
            SymbolDeclarator sb = JavaParserFactory.getSymbolDeclarator(parameter, typeSolver);
            int index = 0;
            for (ValueDeclaration decl : sb.getSymbolDeclarations()) {
                if (decl.getName().equals(name)) {
                    if (getParentNode(wrappedNode) instanceof MethodCallExpr) {
                        MethodCallExpr methodCallExpr = (MethodCallExpr) getParentNode(wrappedNode);
                        MethodUsage methodUsage = JavaParserFacade.get(typeSolver).solveMethodAsUsage(methodCallExpr);
                        int i = pos(methodCallExpr, wrappedNode);
                        Type lambdaType = methodUsage.getParamTypes().get(i);
                        Optional<MethodUsage> functionalMethodOpt = FunctionalInterfaceLogic.getFunctionalMethod(lambdaType);
                        if (functionalMethodOpt.isPresent()) {
                            MethodUsage functionalMethod = functionalMethodOpt.get();
                            InferenceContext inferenceContext = new InferenceContext(MyObjectProvider.INSTANCE);
                            inferenceContext.addPair(lambdaType, new ReferenceTypeImpl(lambdaType.asReferenceType().getTypeDeclaration(), typeSolver));
                            boolean found = false;
                            int lambdaParamIndex;
                            for (lambdaParamIndex = 0; lambdaParamIndex < wrappedNode.getParameters().size(); lambdaParamIndex++) {
                                if (wrappedNode.getParameter(lambdaParamIndex).getName().getIdentifier().equals(name)) {
                                    found = true;
                                    break;
                                }
                            }
                            if (!found) {
                                return Optional.empty();
                            }
                            Type argType = inferenceContext.resolve(inferenceContext.addSingle(functionalMethod.getParamType(lambdaParamIndex)));
                            Value value = new Value(argType.isWildcard() ? LambdaConstraintType.bound(argType.asWildcard().getBoundedType()) : LambdaConstraintType.bound(argType), name);
                            return Optional.of(value);
                        } else {
                            return Optional.empty();
                        }
                    } else if (getParentNode(wrappedNode) instanceof VariableDeclarator) {
                        VariableDeclarator variableDeclarator = (VariableDeclarator) getParentNode(wrappedNode);
                        Type t = JavaParserFacade.get(typeSolver).convertToUsageVariableType(variableDeclarator);
                        Optional<MethodUsage> functionalMethod = FunctionalInterfaceLogic.getFunctionalMethod(t);
                        if (functionalMethod.isPresent()) {
                            Type lambdaType = functionalMethod.get().getParamType(index);
                            Map<TypeParameterDeclaration, Type> inferredTypes = new HashMap<>();
                            if (lambdaType.isReferenceType()) {
                                for (Tuple2<TypeParameterDeclaration, Type> entry : lambdaType.asReferenceType().getTypeParametersMap()) {
                                    if (entry._2.isTypeVariable() && entry._2.asTypeParameter().declaredOnType()) {
                                        Type ot = t.asReferenceType().typeParametersMap().getValue(entry._1);
                                        lambdaType = lambdaType.replaceTypeVariables(entry._1, ot, inferredTypes);
                                    }
                                }
                            } else if (lambdaType.isTypeVariable() && lambdaType.asTypeParameter().declaredOnType()) {
                                lambdaType = t.asReferenceType().typeParametersMap().getValue(lambdaType.asTypeParameter());
                            }
                            Value value = new Value(lambdaType, name);
                            return Optional.of(value);
                        } else {
                            throw new UnsupportedOperationException();
                        }
                    } else {
                        throw new UnsupportedOperationException();
                    }
                }
                index++;
            }
        }
        return getParent().solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        for (Parameter parameter : wrappedNode.getParameters()) {
            SymbolReference<ValueDeclaration> symbolReference = solveWith(JavaParserFactory.getSymbolDeclarator(parameter, typeSolver), name);
            if (symbolReference.isSolved()) {
                return symbolReference;
            }
        }
        return getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        return getParent().solveType(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
    protected final Optional<Value> solveWithAsValue(SymbolDeclarator symbolDeclarator, String name, TypeSolver typeSolver) {
        for (ValueDeclaration decl : symbolDeclarator.getSymbolDeclarations()) {
            if (decl.getName().equals(name)) {
                throw new UnsupportedOperationException();
            }
        }
        return Optional.empty();
    }
    private int pos(MethodCallExpr callExpr, Expression param) {
        int i = 0;
        for (Expression p : callExpr.getArguments()) {
            if (p == param) {
                return i;
            }
            i++;
        }
        throw new IllegalArgumentException();
    }
}
