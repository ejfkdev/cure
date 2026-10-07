package com.github.javaparser.symbolsolver.logic;

import com.github.javaparser.symbolsolver.model.declarations.ClassDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import java.util.ArrayList;
import java.util.List;

public abstract class AbstractClassDeclaration extends AbstractTypeDeclaration implements ClassDeclaration {
    @Override
    public boolean hasName() {
        return getQualifiedName() != null;
    }
    @Override
    public final List<ReferenceType> getAllSuperClasses() {
        List<ReferenceType> superclasses = new ArrayList<>();
        ReferenceType superClass = getSuperClass();
        if (superClass != null) {
            superclasses.add(superClass);
            superclasses.addAll(superClass.getAllClassesAncestors());
        }
        if (superclasses.removeIf((s) -> s.getQualifiedName().equals(Object.class.getCanonicalName()))) {
            superclasses.add(object());
        }
        return superclasses;
    }
    @Override
    public final List<ReferenceType> getAllInterfaces() {
        List<ReferenceType> interfaces = new ArrayList<>();
        for (ReferenceType interfaceDeclaration : getInterfaces()) {
            interfaces.add(interfaceDeclaration);
            interfaces.addAll(interfaceDeclaration.getAllInterfacesAncestors());
        }
        ReferenceType superClass = this.getSuperClass();
        if (superClass != null) {
            interfaces.addAll(superClass.getAllInterfacesAncestors());
        }
        return interfaces;
    }
    @Override
    public final ClassDeclaration asClass() {
        return this;
    }
    protected abstract ReferenceType object();
}
