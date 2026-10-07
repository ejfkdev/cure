package com.github.javaparser.symbolsolver.model.declarations;

public interface ParameterDeclaration extends ValueDeclaration {
    @Override
    default boolean isParameter() {
        return true;
    }
    @Override
    default ParameterDeclaration asParameter() {
        return this;
    }
    boolean isVariadic();
    default String describeType() {
        return isVariadic() ? getType().asArrayType().getComponentType().describe() + "..." : getType().describe();
    }
}
