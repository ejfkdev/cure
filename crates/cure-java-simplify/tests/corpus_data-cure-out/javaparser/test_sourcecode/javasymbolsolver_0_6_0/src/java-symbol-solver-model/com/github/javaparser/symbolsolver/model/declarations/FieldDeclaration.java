package com.github.javaparser.symbolsolver.model.declarations;

public interface FieldDeclaration extends ValueDeclaration, HasAccessLevel {
    boolean isStatic();
    @Override
    default boolean isField() {
        return true;
    }
    @Override
    default FieldDeclaration asField() {
        return this;
    }
    TypeDeclaration declaringType();
}
