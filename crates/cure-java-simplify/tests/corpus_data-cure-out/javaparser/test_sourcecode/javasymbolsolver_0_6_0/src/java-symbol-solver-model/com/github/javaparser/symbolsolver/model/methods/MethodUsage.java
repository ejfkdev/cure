package com.github.javaparser.symbolsolver.model.methods;

import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParametersMap;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParametrized;
import java.util.*;

public class MethodUsage implements TypeParametrized {
    private MethodDeclaration declaration;
    private List<Type> paramTypes = new ArrayList<>();
    private Type returnType;
    private TypeParametersMap typeParametersMap;
    public MethodUsage(MethodDeclaration declaration) {
        this.typeParametersMap = TypeParametersMap.empty();
        this.declaration = declaration;
        for (int i = 0; i < declaration.getNumberOfParams(); i++) {
            paramTypes.add(declaration.getParam(i).getType());
        }
        returnType = declaration.getReturnType();
    }
    public MethodUsage(MethodDeclaration declaration, List<Type> paramTypes, Type returnType) {
        this(declaration, paramTypes, returnType, TypeParametersMap.empty());
    }
    private MethodUsage(MethodDeclaration declaration, List<Type> paramTypes, Type returnType, TypeParametersMap typeParametersMap) {
        this.declaration = declaration;
        this.paramTypes = paramTypes;
        this.returnType = returnType;
        this.typeParametersMap = typeParametersMap;
    }
    @Override
    public String toString() {
        return "MethodUsage{declaration=" + declaration + ", paramTypes=" + paramTypes + '}';
    }
    public MethodDeclaration getDeclaration() {
        return declaration;
    }
    public String getName() {
        return declaration.getName();
    }
    public ReferenceTypeDeclaration declaringType() {
        return declaration.declaringType();
    }
    public Type returnType() {
        return returnType;
    }
    public List<Type> getParamTypes() {
        return paramTypes;
    }
    public MethodUsage replaceParamType(int i, Type replaced) {
        if (paramTypes.get(i) == replaced) {
            return this;
        }
        List<Type> newParams = new LinkedList<>(paramTypes);
        newParams.set(i, replaced);
        return new MethodUsage(declaration, newParams, returnType, typeParametersMap);
    }
    public MethodUsage replaceReturnType(Type returnType) {
        return returnType == this.returnType ? this : new MethodUsage(declaration, paramTypes, returnType, typeParametersMap);
    }
    public int getNoParams() {
        return paramTypes.size();
    }
    public Type getParamType(int i) {
        return paramTypes.get(i);
    }
    public MethodUsage replaceTypeParameter(TypeParameterDeclaration typeParameter, Type type) {
        if (type == null) {
            throw new IllegalArgumentException();
        }
        MethodUsage res = new MethodUsage(declaration, paramTypes, returnType, typeParametersMap.toBuilder().setValue(typeParameter, type).build());
        Map<TypeParameterDeclaration, Type> inferredTypes = new HashMap<>();
        for (int i = 0; i < paramTypes.size(); i++) {
            Type newParamType = paramTypes.get(i).replaceTypeVariables(typeParameter, type, inferredTypes);
            res = res.replaceParamType(i, newParamType);
        }
        Type newReturnType = res.returnType.replaceTypeVariables(typeParameter, type, inferredTypes);
        res = res.replaceReturnType(newReturnType);
        return res;
    }
    @Override
    public TypeParametersMap typeParametersMap() {
        return typeParametersMap;
    }
    public String getQualifiedSignature() {
        return this.getDeclaration().getQualifiedSignature();
    }
}
