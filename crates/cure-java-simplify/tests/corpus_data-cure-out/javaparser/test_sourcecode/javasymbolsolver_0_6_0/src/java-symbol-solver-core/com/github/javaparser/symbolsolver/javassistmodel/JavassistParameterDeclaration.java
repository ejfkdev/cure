package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.model.declarations.ParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import javassist.CtClass;

public class JavassistParameterDeclaration implements ParameterDeclaration {
    private Type type;
    private TypeSolver typeSolver;
    private boolean variadic;
    public JavassistParameterDeclaration(CtClass type, TypeSolver typeSolver, boolean variadic) {
        this(JavassistFactory.typeUsageFor(type, typeSolver), typeSolver, variadic);
    }
    public JavassistParameterDeclaration(Type type, TypeSolver typeSolver, boolean variadic) {
        this.type = type;
        this.typeSolver = typeSolver;
        this.variadic = variadic;
    }
    @Override
    public String toString() {
        return "JavassistParameterDeclaration{type=" + type + ", typeSolver=" + typeSolver + ", variadic=" + variadic + '}';
    }
    @Override
    public String getName() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isField() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isParameter() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isVariadic() {
        return variadic;
    }
    @Override
    public boolean isType() {
        throw new UnsupportedOperationException();
    }
    @Override
    public Type getType() {
        return type;
    }
}
