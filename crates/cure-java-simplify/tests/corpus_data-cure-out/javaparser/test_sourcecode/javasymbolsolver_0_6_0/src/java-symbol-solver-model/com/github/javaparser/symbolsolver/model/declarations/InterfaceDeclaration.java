package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import java.util.ArrayList;
import java.util.List;

public interface InterfaceDeclaration extends ReferenceTypeDeclaration, TypeParametrizable, HasAccessLevel {
    @Override
    default boolean isInterface() {
        return true;
    }
    List<ReferenceType> getInterfacesExtended();
    default List<ReferenceType> getAllInterfacesExtended() {
        List<ReferenceType> interfaces = new ArrayList<>();
        for (ReferenceType interfaceDeclaration : getInterfacesExtended()) {
            interfaces.add(interfaceDeclaration);
            interfaces.addAll(interfaceDeclaration.getAllInterfacesAncestors());
        }
        return interfaces;
    }
}
