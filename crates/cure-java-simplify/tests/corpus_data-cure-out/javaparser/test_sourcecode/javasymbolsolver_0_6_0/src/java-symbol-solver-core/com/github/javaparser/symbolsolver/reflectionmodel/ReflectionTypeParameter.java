package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.model.declarations.MethodLikeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParametrizable;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.lang.reflect.Constructor;
import java.lang.reflect.GenericDeclaration;
import java.lang.reflect.Method;
import java.lang.reflect.TypeVariable;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public class ReflectionTypeParameter implements TypeParameterDeclaration {
    private TypeVariable typeVariable;
    private TypeSolver typeSolver;
    private TypeParametrizable container;
    public ReflectionTypeParameter(TypeVariable typeVariable, boolean declaredOnClass, TypeSolver typeSolver) {
        GenericDeclaration genericDeclaration = typeVariable.getGenericDeclaration();
        if (genericDeclaration instanceof Class) {
            container = ReflectionFactory.typeDeclarationFor((Class) genericDeclaration, typeSolver);
        } else if (genericDeclaration instanceof Method) {
            container = new ReflectionMethodDeclaration((Method) genericDeclaration, typeSolver);
        } else if (genericDeclaration instanceof Constructor) {
            container = new ReflectionConstructorDeclaration((Constructor) genericDeclaration, typeSolver);
        }
        this.typeVariable = typeVariable;
        this.typeSolver = typeSolver;
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (!(o instanceof TypeParameterDeclaration)) 
            return false;
        TypeParameterDeclaration that = (TypeParameterDeclaration) o;
        return !getQualifiedName().equals(that.getQualifiedName()) ? false : declaredOnType() != that.declaredOnType() ? false : declaredOnMethod() == that.declaredOnMethod();
    }
    @Override
    public int hashCode() {
        int result = typeVariable.hashCode();
        result = 31 * result + container.hashCode();
        return result;
    }
    @Override
    public String getName() {
        return typeVariable.getName();
    }
    @Override
    public String getContainerQualifiedName() {
        return ((ReferenceTypeDeclaration) container).getQualifiedName();
    }
    @Override
    public String getContainerId() {
        return ((ReferenceTypeDeclaration) container).getId();
    }
    @Override
    public TypeParametrizable getContainer() {
        return this.container;
    }
    @Override
    public List<Bound> getBounds(TypeSolver typeSolver) {
        return Arrays.stream(typeVariable.getBounds()).map((refB) -> Bound.extendsBound(ReflectionFactory.typeUsageFor(refB, typeSolver))).collect(Collectors.toList());
    }
    @Override
    public String toString() {
        return "ReflectionTypeParameter{typeVariable=" + typeVariable + '}';
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return container instanceof ReferenceTypeDeclaration ? Optional.of((ReferenceTypeDeclaration) container) : Optional.empty();
    }
}
