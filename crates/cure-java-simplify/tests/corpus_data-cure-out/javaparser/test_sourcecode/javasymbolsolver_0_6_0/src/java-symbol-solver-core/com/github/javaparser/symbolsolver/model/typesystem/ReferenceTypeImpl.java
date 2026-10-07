package com.github.javaparser.symbolsolver.model.typesystem;

import com.github.javaparser.symbolsolver.javaparsermodel.LambdaArgumentTypePlaceholder;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserTypeVariableDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration.Bound;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

public class ReferenceTypeImpl extends ReferenceType {
    public static ReferenceType undeterminedParameters(ReferenceTypeDeclaration typeDeclaration, TypeSolver typeSolver) {
        return new ReferenceTypeImpl(typeDeclaration, typeDeclaration.getTypeParameters().stream().map((tp) -> new TypeVariable(tp)).collect(Collectors.toList()), typeSolver);
    }
    @Override
    protected ReferenceType create(ReferenceTypeDeclaration typeDeclaration, List<Type> typeParametersCorrected, TypeSolver typeSolver) {
        return new ReferenceTypeImpl(typeDeclaration, typeParametersCorrected, typeSolver);
    }
    @Override
    protected ReferenceType create(ReferenceTypeDeclaration typeDeclaration, TypeSolver typeSolver) {
        return new ReferenceTypeImpl(typeDeclaration, typeSolver);
    }
    public ReferenceTypeImpl(ReferenceTypeDeclaration typeDeclaration, TypeSolver typeSolver) {
        super(typeDeclaration, typeSolver);
    }
    public ReferenceTypeImpl(ReferenceTypeDeclaration typeDeclaration, List<Type> typeParameters, TypeSolver typeSolver) {
        super(typeDeclaration, typeParameters, typeSolver);
    }
    @Override
    public TypeParameterDeclaration asTypeParameter() {
        if (this.typeDeclaration instanceof JavaParserTypeVariableDeclaration) {
            return ((JavaParserTypeVariableDeclaration) this.typeDeclaration).asTypeParameter();
        }
        throw new UnsupportedOperationException(this.typeDeclaration.getClass().getCanonicalName());
    }
    @Override
    public boolean isAssignableBy(Type other) {
        if (other instanceof NullType) {
            return !this.isPrimitive();
        }
        if (!other.isVoid() && this.getQualifiedName().equals(Object.class.getCanonicalName())) {
            return true;
        }
        if (other.isPrimitive()) {
            return this.getQualifiedName().equals(Object.class.getCanonicalName()) ? true : isCorrespondingBoxingType(other.describe()) ? true : typeSolver.tryToSolveType(other.asPrimitive().getBoxTypeQName()).getCorrespondingDeclaration().canBeAssignedTo(super.typeDeclaration);
        }
        if (other instanceof LambdaArgumentTypePlaceholder) {
            return this.getTypeDeclaration().hasAnnotation(FunctionalInterface.class.getCanonicalName());
        } else if (other instanceof ReferenceTypeImpl) {
            ReferenceTypeImpl otherRef = (ReferenceTypeImpl) other;
            if (compareConsideringTypeParameters(otherRef)) {
                return true;
            }
            for (ReferenceType otherAncestor : otherRef.getAllAncestors()) {
                if (compareConsideringTypeParameters(otherAncestor)) {
                    return true;
                }
            }
            return false;
        } else if (other.isTypeVariable()) {
            for (Bound bound : other.asTypeVariable().asTypeParameter().getBounds(typeSolver)) {
                if (bound.isExtends()) {
                    if (this.isAssignableBy(bound.getType())) {
                        return true;
                    }
                }
            }
            return false;
        } else 
            return other.isConstraint() ? isAssignableBy(other.asConstraintType().getBound()) : other.isWildcard() ? this.getQualifiedName().equals(Object.class.getCanonicalName()) ? true : other.asWildcard().isExtends() ? isAssignableBy(other.asWildcard().getBoundedType()) : false : false;
    }
    @Override
    public Set<MethodUsage> getDeclaredMethods() {
        Set<MethodUsage> methods = new HashSet<>();
        for (MethodDeclaration methodDeclaration : getTypeDeclaration().getDeclaredMethods()) {
            MethodUsage methodUsage = new MethodUsage(methodDeclaration);
            methods.add(methodUsage);
        }
        return methods;
    }
}
