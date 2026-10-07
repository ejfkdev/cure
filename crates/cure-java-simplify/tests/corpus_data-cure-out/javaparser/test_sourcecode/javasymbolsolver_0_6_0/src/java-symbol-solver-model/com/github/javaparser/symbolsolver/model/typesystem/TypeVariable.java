package com.github.javaparser.symbolsolver.model.typesystem;

import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import java.util.Map;

public class TypeVariable implements Type {
    private TypeParameterDeclaration typeParameter;
    public TypeVariable(TypeParameterDeclaration typeParameter) {
        this.typeParameter = typeParameter;
    }
    @Override
    public String toString() {
        return "TypeVariable {" + typeParameter.getQualifiedName() + "}";
    }
    public String qualifiedName() {
        return this.typeParameter.getQualifiedName();
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        TypeVariable that = (TypeVariable) o;
        return !typeParameter.getName().equals(that.typeParameter.getName()) ? false : typeParameter.declaredOnType() != that.typeParameter.declaredOnType() ? false : typeParameter.declaredOnMethod() == that.typeParameter.declaredOnMethod();
    }
    @Override
    public int hashCode() {
        return typeParameter.hashCode();
    }
    @Override
    public boolean isArray() {
        return false;
    }
    @Override
    public boolean isPrimitive() {
        return false;
    }
    @Override
    public Type replaceTypeVariables(TypeParameterDeclaration tpToBeReplaced, Type replaced, Map<TypeParameterDeclaration, Type> inferredTypes) {
        if (tpToBeReplaced.getName().equals(this.typeParameter.getName())) {
            inferredTypes.put(this.asTypeParameter(), replaced);
            return replaced;
        } else {
            return this;
        }
    }
    @Override
    public boolean isReferenceType() {
        return false;
    }
    @Override
    public String describe() {
        return typeParameter.getName();
    }
    @Override
    public TypeParameterDeclaration asTypeParameter() {
        return typeParameter;
    }
    @Override
    public TypeVariable asTypeVariable() {
        return this;
    }
    @Override
    public boolean isTypeVariable() {
        return true;
    }
    @Override
    public boolean isAssignableBy(Type other) {
        return other.isTypeVariable() ? describe().equals(other.describe()) : true;
    }
}
