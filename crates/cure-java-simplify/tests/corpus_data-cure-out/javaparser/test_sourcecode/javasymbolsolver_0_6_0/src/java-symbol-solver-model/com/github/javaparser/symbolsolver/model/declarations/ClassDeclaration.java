package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import java.util.List;

public interface ClassDeclaration extends ReferenceTypeDeclaration, TypeParametrizable, HasAccessLevel {
    @Override
    default boolean isClass() {
        return true;
    }
    ReferenceType getSuperClass();
    List<ReferenceType> getInterfaces();
    List<ReferenceType> getAllSuperClasses();
    List<ReferenceType> getAllInterfaces();
    List<ConstructorDeclaration> getConstructors();
}
