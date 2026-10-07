package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.typesystem.Type;

public interface ValueDeclaration extends Declaration {
    Type getType();
}
