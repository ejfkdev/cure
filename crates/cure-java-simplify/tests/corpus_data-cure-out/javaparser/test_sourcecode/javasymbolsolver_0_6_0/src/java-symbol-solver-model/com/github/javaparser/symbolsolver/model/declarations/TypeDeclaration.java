package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import java.util.Optional;
import java.util.Set;

public interface TypeDeclaration extends Declaration {
    default Set<ReferenceTypeDeclaration> internalTypes() {
        throw new UnsupportedOperationException("InternalTypes not available for " + this.getClass().getCanonicalName());
    }
    default ReferenceTypeDeclaration getInternalType(String name) {
        return this.internalTypes().stream().filter((f) -> f.getName().equals(name)).findFirst().orElseThrow(() -> new UnsolvedSymbolException("Internal type not found: " + name));
    }
    default boolean hasInternalType(String name) {
        return this.internalTypes().stream().anyMatch((f) -> f.getName().equals(name));
    }
    Optional<ReferenceTypeDeclaration> containerType();
    default boolean isClass() {
        return false;
    }
    default boolean isInterface() {
        return false;
    }
    default boolean isEnum() {
        return false;
    }
    default boolean isTypeParameter() {
        return false;
    }
    @Override
    default boolean isType() {
        return true;
    }
    @Override
    default TypeDeclaration asType() {
        return this;
    }
    default ClassDeclaration asClass() {
        throw new UnsupportedOperationException(String.format("%s is not a class", this));
    }
    default InterfaceDeclaration asInterface() {
        throw new UnsupportedOperationException(String.format("%s is not an interface", this));
    }
    default EnumDeclaration asEnum() {
        throw new UnsupportedOperationException(String.format("%s is not an enum", this));
    }
    default TypeParameterDeclaration asTypeParameter() {
        throw new UnsupportedOperationException(String.format("%s is not a type parameter", this));
    }
    default ReferenceTypeDeclaration asReferenceType() {
        throw new UnsupportedOperationException(String.format("%s is not a reference type", this));
    }
    String getPackageName();
    String getClassName();
    String getQualifiedName();
    default String getId() {
        String qname = getQualifiedName();
        return qname == null ? String.format("<localClass>:%s", getName()) : qname;
    }
}
