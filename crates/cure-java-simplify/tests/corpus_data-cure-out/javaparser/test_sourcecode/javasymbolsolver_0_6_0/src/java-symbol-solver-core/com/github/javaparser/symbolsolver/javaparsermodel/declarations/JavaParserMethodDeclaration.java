package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.ObjectCreationExpr;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.declarations.common.MethodDeclarationCommonLogic;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.List;
import java.util.stream.Collectors;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class JavaParserMethodDeclaration implements MethodDeclaration {
    private com.github.javaparser.ast.body.MethodDeclaration wrappedNode;
    private TypeSolver typeSolver;
    public JavaParserMethodDeclaration(com.github.javaparser.ast.body.MethodDeclaration wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public String toString() {
        return "JavaParserMethodDeclaration{wrappedNode=" + wrappedNode + ", typeSolver=" + typeSolver + '}';
    }
    @Override
    public ReferenceTypeDeclaration declaringType() {
        return getParentNode(wrappedNode) instanceof ObjectCreationExpr ? new JavaParserAnonymousClassDeclaration((ObjectCreationExpr) getParentNode(wrappedNode), typeSolver) : JavaParserFactory.toTypeDeclaration(getParentNode(wrappedNode), typeSolver);
    }
    @Override
    public Type getReturnType() {
        return JavaParserFacade.get(typeSolver).convert(wrappedNode.getType(), getContext());
    }
    @Override
    public int getNumberOfParams() {
        return wrappedNode.getParameters().size();
    }
    @Override
    public ParameterDeclaration getParam(int i) {
        if (i < 0 || i >= getNumberOfParams()) {
            throw new IllegalArgumentException(String.format("No param with index %d. Number of params: %d", i, getNumberOfParams()));
        }
        return new JavaParserParameterDeclaration(wrappedNode.getParameters().get(i), typeSolver);
    }
    public MethodUsage getUsage(Node node) {
        throw new UnsupportedOperationException();
    }
    public MethodUsage resolveTypeVariables(Context context, List<Type> parameterTypes) {
        return new MethodDeclarationCommonLogic(this, typeSolver).resolveTypeVariables(context, parameterTypes);
    }
    private Context getContext() {
        return JavaParserFactory.getContext(wrappedNode, typeSolver);
    }
    @Override
    public boolean isAbstract() {
        return !wrappedNode.getBody().isPresent();
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    @Override
    public boolean isField() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isParameter() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isType() {
        throw new UnsupportedOperationException();
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return this.wrappedNode.getTypeParameters().stream().map((astTp) -> new JavaParserTypeParameter(astTp, typeSolver)).collect(Collectors.toList());
    }
    @Override
    public boolean isDefaultMethod() {
        return wrappedNode.isDefault();
    }
    @Override
    public boolean isStatic() {
        return wrappedNode.isStatic();
    }
    public com.github.javaparser.ast.body.MethodDeclaration getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public AccessLevel accessLevel() {
        return Helper.toAccessLevel(wrappedNode.getModifiers());
    }
}
