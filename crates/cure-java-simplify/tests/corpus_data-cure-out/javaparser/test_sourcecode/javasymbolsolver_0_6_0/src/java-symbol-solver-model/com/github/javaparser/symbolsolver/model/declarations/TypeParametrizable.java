package com.github.javaparser.symbolsolver.model.declarations;

import java.util.List;
import java.util.Optional;

public interface TypeParametrizable {
    List<TypeParameterDeclaration> getTypeParameters();
    Optional<TypeParameterDeclaration> findTypeParameter(String name);
}
