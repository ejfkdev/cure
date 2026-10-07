package com.github.javaparser.symbolsolver.model.declarations;

public interface EnumDeclaration extends ReferenceTypeDeclaration, HasAccessLevel {
    @Override
    default boolean isEnum() {
        return true;
    }
    @Override
    default EnumDeclaration asEnum() {
        return this;
    }
}
