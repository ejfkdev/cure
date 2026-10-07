package com.github.javaparser.symbolsolver.model.declarations;

public interface Declaration {
    default boolean hasName() {
        return true;
    }
    String getName();
    default boolean isField() {
        return false;
    }
    default boolean isParameter() {
        return false;
    }
    default boolean isType() {
        return false;
    }
    default boolean isMethod() {
        return false;
    }
    default FieldDeclaration asField() {
        throw new UnsupportedOperationException(String.format("%s is not a FieldDeclaration", this));
    }
    default ParameterDeclaration asParameter() {
        throw new UnsupportedOperationException(String.format("%s is not a ParameterDeclaration", this));
    }
    default TypeDeclaration asType() {
        throw new UnsupportedOperationException(String.format("%s is not a TypeDeclaration", this));
    }
    default MethodDeclaration asMethod() {
        throw new UnsupportedOperationException(String.format("%s is not a MethodDeclaration", this));
    }
}
