package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.LambdaExpr;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ArrayType;
import com.github.javaparser.symbolsolver.model.typesystem.PrimitiveType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class JavaParserSymbolDeclaration implements ValueDeclaration {
    private String name;
    private Node wrappedNode;
    private boolean field;
    private boolean parameter;
    private boolean variable;
    private TypeSolver typeSolver;
    private JavaParserSymbolDeclaration(Node wrappedNode, String name, TypeSolver typeSolver, boolean field, boolean parameter, boolean variable) {
        this.name = name;
        this.wrappedNode = wrappedNode;
        this.field = field;
        this.variable = variable;
        this.parameter = parameter;
        this.typeSolver = typeSolver;
    }
    public static JavaParserFieldDeclaration field(VariableDeclarator wrappedNode, TypeSolver typeSolver) {
        return new JavaParserFieldDeclaration(wrappedNode, typeSolver);
    }
    public static JavaParserParameterDeclaration parameter(Parameter parameter, TypeSolver typeSolver) {
        return new JavaParserParameterDeclaration(parameter, typeSolver);
    }
    public static JavaParserSymbolDeclaration localVar(VariableDeclarator variableDeclarator, TypeSolver typeSolver) {
        return new JavaParserSymbolDeclaration(variableDeclarator, variableDeclarator.getName().getId(), typeSolver, false, false, true);
    }
    public static int getParamPos(Parameter parameter) {
        int pos = 0;
        for (Node node : getParentNode(parameter).getChildNodes()) {
            if (node == parameter) {
                return pos;
            } else if (node instanceof Parameter) {
                pos++;
            }
        }
        return pos;
    }
    public static int getParamPos(Node node) {
        if (getParentNode(node) instanceof MethodCallExpr) {
            MethodCallExpr call = (MethodCallExpr) getParentNode(node);
            for (int i = 0; i < call.getArguments().size(); i++) {
                if (call.getArguments().get(i) == node) 
                    return i;
            }
            throw new IllegalStateException();
        } else {
            throw new IllegalArgumentException();
        }
    }
    @Override
    public String toString() {
        return "JavaParserSymbolDeclaration{name='" + name + '\'' + ", wrappedNode=" + wrappedNode + '}';
    }
    @Override
    public String getName() {
        return name;
    }
    @Override
    public boolean isField() {
        return field;
    }
    @Override
    public boolean isParameter() {
        return parameter;
    }
    @Override
    public boolean isType() {
        return false;
    }
    @Override
    public Type getType() {
        if (wrappedNode instanceof Parameter) {
            Parameter parameter = (Parameter) wrappedNode;
            if (getParentNode(wrappedNode) instanceof LambdaExpr) {
                int pos = getParamPos(parameter);
                Type lambdaType = JavaParserFacade.get(typeSolver).getType(getParentNode(wrappedNode));
                throw new UnsupportedOperationException(wrappedNode.getClass().getCanonicalName());
            } else {
                Type rawType = parameter.getType() instanceof com.github.javaparser.ast.type.PrimitiveType ? PrimitiveType.byName(((com.github.javaparser.ast.type.PrimitiveType) parameter.getType()).getType().name()) : JavaParserFacade.get(typeSolver).convertToUsage(parameter.getType(), wrappedNode);
                return parameter.isVarArgs() ? new ArrayType(rawType) : rawType;
            }
        } else if (wrappedNode instanceof VariableDeclarator) {
            VariableDeclarator variableDeclarator = (VariableDeclarator) wrappedNode;
            if (getParentNode(wrappedNode) instanceof VariableDeclarationExpr) {
                return JavaParserFacade.get(typeSolver).convert(variableDeclarator.getType(), JavaParserFactory.getContext(wrappedNode, typeSolver));
            } else if (getParentNode(wrappedNode) instanceof FieldDeclaration) {
                return JavaParserFacade.get(typeSolver).convert(variableDeclarator.getType(), JavaParserFactory.getContext(wrappedNode, typeSolver));
            } else {
                throw new UnsupportedOperationException(getParentNode(wrappedNode).getClass().getCanonicalName());
            }
        } else {
            throw new UnsupportedOperationException(wrappedNode.getClass().getCanonicalName());
        }
    }
    @Override
    public TypeDeclaration asType() {
        throw new UnsupportedOperationException(this.getClass().getCanonicalName() + ": wrapping " + this.getWrappedNode().getClass().getCanonicalName());
    }
    public Node getWrappedNode() {
        return wrappedNode;
    }
}
