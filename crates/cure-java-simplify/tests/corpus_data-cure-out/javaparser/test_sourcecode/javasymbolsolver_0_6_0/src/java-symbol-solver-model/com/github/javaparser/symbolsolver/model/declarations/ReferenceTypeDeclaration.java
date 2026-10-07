package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

public interface ReferenceTypeDeclaration extends TypeDeclaration, TypeParametrizable {
    @Override
    default ReferenceTypeDeclaration asReferenceType() {
        return this;
    }
    List<ReferenceType> getAncestors();
    default List<ReferenceType> getAllAncestors() {
        List<ReferenceType> ancestors = new ArrayList<>();
        if (!Object.class.getCanonicalName().equals(getQualifiedName())) {
            for (ReferenceType ancestor : getAncestors()) {
                ancestors.add(ancestor);
                for (ReferenceType inheritedAncestor : ancestor.getAllAncestors()) {
                    if (!ancestors.contains(inheritedAncestor)) {
                        ancestors.add(inheritedAncestor);
                    }
                }
            }
        }
        return ancestors;
    }
    default FieldDeclaration getField(String name) {
        Optional<FieldDeclaration> field = this.getAllFields().stream().filter((f) -> f.getName().equals(name)).findFirst();
        if (field.isPresent()) {
            return field.get();
        } else {
            throw new UnsolvedSymbolException("Field not found: " + name);
        }
    }
    default FieldDeclaration getVisibleField(String name) {
        Optional<FieldDeclaration> field = getVisibleFields().stream().filter((f) -> f.getName().equals(name)).findFirst();
        if (field.isPresent()) {
            return field.get();
        } else {
            throw new IllegalArgumentException();
        }
    }
    default boolean hasField(String name) {
        return this.getAllFields().stream().filter((f) -> f.getName().equals(name)).findFirst().isPresent();
    }
    default boolean hasVisibleField(String name) {
        return getVisibleFields().stream().filter((f) -> f.getName().equals(name)).findFirst().isPresent();
    }
    List<FieldDeclaration> getAllFields();
    default List<FieldDeclaration> getVisibleFields() {
        return getAllFields().stream().filter((f) -> f.declaringType().equals(this) || f.accessLevel() != AccessLevel.PRIVATE).collect(Collectors.toList());
    }
    default List<FieldDeclaration> getAllNonStaticFields() {
        return getAllFields().stream().filter((it) -> !it.isStatic()).collect(Collectors.toList());
    }
    default List<FieldDeclaration> getAllStaticFields() {
        return getAllFields().stream().filter((it) -> it.isStatic()).collect(Collectors.toList());
    }
    default List<FieldDeclaration> getDeclaredFields() {
        return getAllFields().stream().filter((it) -> it.declaringType().getQualifiedName().equals(getQualifiedName())).collect(Collectors.toList());
    }
    Set<MethodDeclaration> getDeclaredMethods();
    Set<MethodUsage> getAllMethods();
    boolean isAssignableBy(Type type);
    default boolean canBeAssignedTo(ReferenceTypeDeclaration other) {
        return other.isAssignableBy(this);
    }
    boolean isAssignableBy(ReferenceTypeDeclaration other);
    boolean hasDirectlyAnnotation(String qualifiedName);
    default boolean hasAnnotation(String qualifiedName) {
        return hasDirectlyAnnotation(qualifiedName) ? true : getAllAncestors().stream().anyMatch((it) -> it.asReferenceType().getTypeDeclaration().hasDirectlyAnnotation(qualifiedName));
    }
    boolean isFunctionalInterface();
    @Override
    default Optional<TypeParameterDeclaration> findTypeParameter(String name) {
        for (TypeParameterDeclaration tp : this.getTypeParameters()) {
            if (tp.getName().equals(name)) {
                return Optional.of(tp);
            }
        }
        return this.containerType().isPresent() ? this.containerType().get().findTypeParameter(name) : Optional.empty();
    }
}
