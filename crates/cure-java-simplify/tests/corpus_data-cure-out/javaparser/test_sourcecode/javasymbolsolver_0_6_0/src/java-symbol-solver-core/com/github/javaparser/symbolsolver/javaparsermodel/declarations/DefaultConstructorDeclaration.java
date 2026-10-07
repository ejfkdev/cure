package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.symbolsolver.model.declarations.*;
import java.util.Collections;
import java.util.List;

class DefaultConstructorDeclaration implements ConstructorDeclaration {
    private ClassDeclaration classDeclaration;
    DefaultConstructorDeclaration(ClassDeclaration classDeclaration) {
        this.classDeclaration = classDeclaration;
    }
    @Override
    public ClassDeclaration declaringType() {
        return classDeclaration;
    }
    @Override
    public int getNumberOfParams() {
        return 0;
    }
    @Override
    public ParameterDeclaration getParam(int i) {
        throw new UnsupportedOperationException();
    }
    @Override
    public String getName() {
        return classDeclaration.getName();
    }
    @Override
    public AccessLevel accessLevel() {
        return AccessLevel.PUBLIC;
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return Collections.emptyList();
    }
}
