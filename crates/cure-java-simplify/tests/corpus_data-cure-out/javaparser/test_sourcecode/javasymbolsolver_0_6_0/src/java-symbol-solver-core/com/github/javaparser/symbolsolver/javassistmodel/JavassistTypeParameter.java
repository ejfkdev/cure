package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.model.declarations.MethodLikeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParametrizable;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import javassist.bytecode.SignatureAttribute;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public class JavassistTypeParameter implements TypeParameterDeclaration {
    private SignatureAttribute.TypeParameter wrapped;
    private TypeSolver typeSolver;
    private TypeParametrizable container;
    public JavassistTypeParameter(SignatureAttribute.TypeParameter wrapped, TypeParametrizable container, TypeSolver typeSolver) {
        this.wrapped = wrapped;
        this.typeSolver = typeSolver;
        this.container = container;
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
    public String toString() {
        return "JavassistTypeParameter{" + wrapped.getName() + '}';
    }
    @Override
    public String getName() {
        return wrapped.getName();
    }
    @Override
    public String getContainerQualifiedName() {
        if (this.container instanceof ReferenceTypeDeclaration) {
            return ((ReferenceTypeDeclaration) this.container).getQualifiedName();
        } else if (this.container instanceof MethodLikeDeclaration) {
            return ((MethodLikeDeclaration) this.container).getQualifiedName();
        }
        throw new UnsupportedOperationException();
    }
    @Override
    public String getContainerId() {
        return getContainerQualifiedName();
    }
    @Override
    public TypeParametrizable getContainer() {
        return this.container;
    }
    @Override
    public List<TypeParameterDeclaration.Bound> getBounds(TypeSolver typeSolver) {
        List<Bound> bounds = new ArrayList<>();
        if (wrapped.getClassBound() != null && !wrapped.getClassBound().toString().equals(Object.class.getCanonicalName())) {
            throw new UnsupportedOperationException(wrapped.getClassBound().toString());
        }
        for (SignatureAttribute.ObjectType ot : wrapped.getInterfaceBound()) {
            throw new UnsupportedOperationException(ot.toString());
        }
        return bounds;
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return container instanceof ReferenceTypeDeclaration ? Optional.of((ReferenceTypeDeclaration) container) : Optional.empty();
    }
}
