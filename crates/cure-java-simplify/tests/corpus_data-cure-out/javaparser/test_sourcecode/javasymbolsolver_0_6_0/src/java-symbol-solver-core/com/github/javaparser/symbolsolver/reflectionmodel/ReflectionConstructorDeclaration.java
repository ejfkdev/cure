package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.lang.reflect.Constructor;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

public class ReflectionConstructorDeclaration implements ConstructorDeclaration {
    private Constructor<?> constructor;
    private TypeSolver typeSolver;
    public ReflectionConstructorDeclaration(Constructor<?> constructor, TypeSolver typeSolver) {
        this.constructor = constructor;
        this.typeSolver = typeSolver;
    }
    @Override
    public ClassDeclaration declaringType() {
        return new ReflectionClassDeclaration(constructor.getDeclaringClass(), typeSolver);
    }
    @Override
    public int getNumberOfParams() {
        return constructor.getParameterCount();
    }
    @Override
    public ParameterDeclaration getParam(int i) {
        if (i < 0 || i >= getNumberOfParams()) {
            throw new IllegalArgumentException(String.format("No param with index %d. Number of params: %d", i, getNumberOfParams()));
        }
        boolean variadic = false;
        if (constructor.isVarArgs()) {
            variadic = i == constructor.getParameterCount() - 1;
        }
        return new ReflectionParameterDeclaration(constructor.getParameterTypes()[i], constructor.getGenericParameterTypes()[i], typeSolver, variadic);
    }
    @Override
    public String getName() {
        return constructor.getName();
    }
    @Override
    public AccessLevel accessLevel() {
        return ReflectionFactory.modifiersToAccessLevel(constructor.getModifiers());
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return Arrays.stream(constructor.getTypeParameters()).map((refTp) -> new ReflectionTypeParameter(refTp, false, typeSolver)).collect(Collectors.toList());
    }
}
