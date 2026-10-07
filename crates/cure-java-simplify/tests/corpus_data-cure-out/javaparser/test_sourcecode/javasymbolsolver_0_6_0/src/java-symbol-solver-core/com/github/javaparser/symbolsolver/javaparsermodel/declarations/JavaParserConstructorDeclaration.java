package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.List;
import java.util.stream.Collectors;

public class JavaParserConstructorDeclaration implements ConstructorDeclaration {
    private ClassDeclaration classDeclaration;
    private com.github.javaparser.ast.body.ConstructorDeclaration wrappedNode;
    private TypeSolver typeSolver;
    public JavaParserConstructorDeclaration(ClassDeclaration classDeclaration, com.github.javaparser.ast.body.ConstructorDeclaration wrappedNode, TypeSolver typeSolver) {
        this.classDeclaration = classDeclaration;
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public ClassDeclaration declaringType() {
        return classDeclaration;
    }
    @Override
    public int getNumberOfParams() {
        return this.wrappedNode.getParameters().size();
    }
    @Override
    public ParameterDeclaration getParam(int i) {
        if (i < 0 || i >= getNumberOfParams()) {
            throw new IllegalArgumentException(String.format("No param with index %d. Number of params: %d", i, getNumberOfParams()));
        }
        return new JavaParserParameterDeclaration(wrappedNode.getParameters().get(i), typeSolver);
    }
    @Override
    public String getName() {
        return this.classDeclaration.getName();
    }
    public com.github.javaparser.ast.body.ConstructorDeclaration getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public AccessLevel accessLevel() {
        return Helper.toAccessLevel(wrappedNode.getModifiers());
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return this.wrappedNode.getTypeParameters().stream().map((astTp) -> new JavaParserTypeParameter(astTp, typeSolver)).collect(Collectors.toList());
    }
}
