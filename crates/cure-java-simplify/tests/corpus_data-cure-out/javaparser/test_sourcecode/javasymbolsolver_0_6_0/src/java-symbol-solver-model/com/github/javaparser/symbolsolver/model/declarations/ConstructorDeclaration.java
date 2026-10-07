package com.github.javaparser.symbolsolver.model.declarations;

public interface ConstructorDeclaration extends MethodLikeDeclaration {
    @Override ClassDeclaration declaringType();
}
