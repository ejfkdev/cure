package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.model.declarations.ParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;

public class ReflectionParameterDeclaration implements ParameterDeclaration {
    private Class<?> type;
    private java.lang.reflect.Type genericType;
    private TypeSolver typeSolver;
    private boolean variadic;
    public ReflectionParameterDeclaration(Class<?> type, java.lang.reflect.Type genericType, TypeSolver typeSolver, boolean variadic) {
        this.type = type;
        this.genericType = genericType;
        this.typeSolver = typeSolver;
        this.variadic = variadic;
    }
    @Override
    public String getName() {
        throw new UnsupportedOperationException();
    }
    @Override
    public String toString() {
        return "ReflectionParameterDeclaration{type=" + type + '}';
    }
    @Override
    public boolean isField() {
        return false;
    }
    @Override
    public boolean isParameter() {
        return true;
    }
    @Override
    public boolean isVariadic() {
        return variadic;
    }
    @Override
    public boolean isType() {
        return false;
    }
    @Override
    public Type getType() {
        return ReflectionFactory.typeUsageFor(genericType, typeSolver);
    }
}
