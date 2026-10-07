package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.declarations.common.MethodDeclarationCommonLogic;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

public class ReflectionMethodDeclaration implements MethodDeclaration {
    private Method method;
    private TypeSolver typeSolver;
    public ReflectionMethodDeclaration(Method method, TypeSolver typeSolver) {
        this.method = method;
        if (method.isSynthetic() || method.isBridge()) {
            throw new IllegalArgumentException();
        }
        this.typeSolver = typeSolver;
    }
    @Override
    public String getName() {
        return method.getName();
    }
    @Override
    public boolean isField() {
        return false;
    }
    @Override
    public boolean isParameter() {
        return false;
    }
    @Override
    public String toString() {
        return "ReflectionMethodDeclaration{method=" + method + '}';
    }
    @Override
    public boolean isType() {
        return false;
    }
    @Override
    public ReferenceTypeDeclaration declaringType() {
        return method.getDeclaringClass().isInterface() ? new ReflectionInterfaceDeclaration(method.getDeclaringClass(), typeSolver) : method.getDeclaringClass().isEnum() ? new ReflectionEnumDeclaration(method.getDeclaringClass(), typeSolver) : new ReflectionClassDeclaration(method.getDeclaringClass(), typeSolver);
    }
    @Override
    public Type getReturnType() {
        return ReflectionFactory.typeUsageFor(method.getGenericReturnType(), typeSolver);
    }
    @Override
    public int getNumberOfParams() {
        return method.getParameterTypes().length;
    }
    @Override
    public ParameterDeclaration getParam(int i) {
        boolean variadic = false;
        if (method.isVarArgs()) {
            variadic = i == method.getParameterCount() - 1;
        }
        return new ReflectionParameterDeclaration(method.getParameterTypes()[i], method.getGenericParameterTypes()[i], typeSolver, variadic);
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return Arrays.stream(method.getTypeParameters()).map((refTp) -> new ReflectionTypeParameter(refTp, false, typeSolver)).collect(Collectors.toList());
    }
    public MethodUsage resolveTypeVariables(Context context, List<Type> parameterTypes) {
        return new MethodDeclarationCommonLogic(this, typeSolver).resolveTypeVariables(context, parameterTypes);
    }
    @Override
    public boolean isAbstract() {
        return Modifier.isAbstract(method.getModifiers());
    }
    @Override
    public boolean isDefaultMethod() {
        return method.isDefault();
    }
    @Override
    public boolean isStatic() {
        return Modifier.isStatic(method.getModifiers());
    }
    @Override
    public AccessLevel accessLevel() {
        return ReflectionFactory.modifiersToAccessLevel(this.method.getModifiers());
    }
}
