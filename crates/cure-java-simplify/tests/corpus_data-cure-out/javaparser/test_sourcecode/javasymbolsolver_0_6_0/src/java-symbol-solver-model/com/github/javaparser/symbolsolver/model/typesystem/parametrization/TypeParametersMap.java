package com.github.javaparser.symbolsolver.model.typesystem.parametrization;

import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.TypeVariable;
import java.util.*;

public class TypeParametersMap {
    public static class Builder {
        private Map<String, Type> nameToValue;
        private Map<String, TypeParameterDeclaration> nameToDeclaration;
        public Builder() {
            nameToValue = new HashMap<>();
            nameToDeclaration = new HashMap<>();
        }
        private Builder(Map<String, Type> nameToValue, Map<String, TypeParameterDeclaration> nameToDeclaration) {
            this.nameToValue = new HashMap<>();
            this.nameToValue.putAll(nameToValue);
            this.nameToDeclaration = new HashMap<>();
            this.nameToDeclaration.putAll(nameToDeclaration);
        }
        public TypeParametersMap build() {
            return new TypeParametersMap(nameToValue, nameToDeclaration);
        }
        public Builder setValue(TypeParameterDeclaration typeParameter, Type value) {
            String qualifiedName = typeParameter.getQualifiedName();
            nameToValue.put(qualifiedName, value);
            nameToDeclaration.put(qualifiedName, typeParameter);
            return this;
        }
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (!(o instanceof TypeParametersMap)) 
            return false;
        TypeParametersMap that = (TypeParametersMap) o;
        return nameToValue.equals(that.nameToValue) && nameToDeclaration.equals(that.nameToDeclaration);
    }
    @Override
    public int hashCode() {
        return nameToValue.hashCode();
    }
    @Override
    public String toString() {
        return "TypeParametersMap{nameToValue=" + nameToValue + '}';
    }
    private Map<String, Type> nameToValue;
    private Map<String, TypeParameterDeclaration> nameToDeclaration;
    public static TypeParametersMap empty() {
        return new Builder().build();
    }
    private TypeParametersMap(Map<String, Type> nameToValue, Map<String, TypeParameterDeclaration> nameToDeclaration) {
        this.nameToValue = new HashMap<>();
        this.nameToValue.putAll(nameToValue);
        this.nameToDeclaration = new HashMap<>();
        this.nameToDeclaration.putAll(nameToDeclaration);
    }
    public Type getValue(TypeParameterDeclaration typeParameter) {
        String qualifiedName = typeParameter.getQualifiedName();
        return nameToValue.containsKey(qualifiedName) ? nameToValue.get(qualifiedName) : new TypeVariable(typeParameter);
    }
    public Optional<Type> getValueBySignature(String signature) {
        return nameToValue.containsKey(signature) ? Optional.of(nameToValue.get(signature)) : Optional.empty();
    }
    public List<String> getNames() {
        return new ArrayList<>(nameToValue.keySet());
    }
    public List<Type> getTypes() {
        return new ArrayList<>(nameToValue.values());
    }
    public Builder toBuilder() {
        return new Builder(nameToValue, nameToDeclaration);
    }
    public boolean isEmpty() {
        return nameToValue.isEmpty();
    }
    public Type replaceAll(Type type) {
        Map<TypeParameterDeclaration, Type> inferredTypes = new HashMap<>();
        for (TypeParameterDeclaration typeParameterDeclaration : this.nameToDeclaration.values()) {
            type = type.replaceTypeVariables(typeParameterDeclaration, getValue(typeParameterDeclaration), inferredTypes);
        }
        return type;
    }
}
