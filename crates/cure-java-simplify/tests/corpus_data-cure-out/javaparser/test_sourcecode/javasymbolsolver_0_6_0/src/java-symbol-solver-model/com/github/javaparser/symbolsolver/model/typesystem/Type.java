package com.github.javaparser.symbolsolver.model.typesystem;

import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import java.util.HashMap;
import java.util.Map;

public interface Type {
    default boolean isArray() {
        return false;
    }
    default int arrayLevel() {
        return isArray() ? 1 + this.asArrayType().getComponentType().arrayLevel() : 0;
    }
    default boolean isPrimitive() {
        return false;
    }
    default boolean isNull() {
        return false;
    }
    default boolean isReference() {
        return isReferenceType() || isArray() || isTypeVariable() || isNull() || isWildcard();
    }
    default boolean isConstraint() {
        return false;
    }
    default boolean isReferenceType() {
        return false;
    }
    default boolean isVoid() {
        return false;
    }
    default boolean isTypeVariable() {
        return false;
    }
    default boolean isWildcard() {
        return false;
    }
    default ArrayType asArrayType() {
        throw new UnsupportedOperationException(String.format("%s is not an Array", this));
    }
    default ReferenceType asReferenceType() {
        throw new UnsupportedOperationException(String.format("%s is not a Reference Type", this));
    }
    default TypeParameterDeclaration asTypeParameter() {
        throw new UnsupportedOperationException(String.format("%s is not a Type parameter", this));
    }
    default TypeVariable asTypeVariable() {
        throw new UnsupportedOperationException(String.format("%s is not a Type variable", this));
    }
    default PrimitiveType asPrimitive() {
        throw new UnsupportedOperationException(String.format("%s is not a Primitive type", this));
    }
    default Wildcard asWildcard() {
        throw new UnsupportedOperationException(String.format("%s is not a Wildcard", this));
    }
    default LambdaConstraintType asConstraintType() {
        throw new UnsupportedOperationException(String.format("%s is not a constraint type", this));
    }
    String describe();
    default Type replaceTypeVariables(TypeParameterDeclaration tp, Type replaced, Map<TypeParameterDeclaration, Type> inferredTypes) {
        return this;
    }
    default Type replaceTypeVariables(TypeParameterDeclaration tp, Type replaced) {
        return replaceTypeVariables(tp, replaced, new HashMap<>());
    }
    boolean isAssignableBy(Type other);
}
