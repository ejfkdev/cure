package com.github.javaparser.symbolsolver.model.typesystem;

import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import java.util.Map;

public class ArrayType implements Type {
    private Type baseType;
    public ArrayType(Type baseType) {
        this.baseType = baseType;
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        ArrayType that = (ArrayType) o;
        return baseType.equals(that.baseType);
    }
    @Override
    public int hashCode() {
        return baseType.hashCode();
    }
    @Override
    public String toString() {
        return "ArrayTypeUsage{" + baseType + "}";
    }
    @Override
    public ArrayType asArrayType() {
        return this;
    }
    @Override
    public boolean isArray() {
        return true;
    }
    @Override
    public String describe() {
        return baseType.describe() + "[]";
    }
    public Type getComponentType() {
        return baseType;
    }
    @Override
    public boolean isAssignableBy(Type other) {
        if (other.isArray()) {
            return baseType.isPrimitive() && other.asArrayType().getComponentType().isPrimitive() ? baseType.equals(other.asArrayType().getComponentType()) : baseType.isAssignableBy(other.asArrayType().getComponentType());
        } else if (other.isNull()) {
            return true;
        }
        return false;
    }
    @Override
    public Type replaceTypeVariables(TypeParameterDeclaration tpToReplace, Type replaced, Map<TypeParameterDeclaration, Type> inferredTypes) {
        Type baseTypeReplaced = baseType.replaceTypeVariables(tpToReplace, replaced, inferredTypes);
        return baseTypeReplaced == baseType ? this : new ArrayType(baseTypeReplaced);
    }
}
