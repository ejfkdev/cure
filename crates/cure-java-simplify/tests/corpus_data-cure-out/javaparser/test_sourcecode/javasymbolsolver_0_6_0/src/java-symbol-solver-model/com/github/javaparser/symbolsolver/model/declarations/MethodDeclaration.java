package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.typesystem.Type;

public interface MethodDeclaration extends MethodLikeDeclaration {
    Type getReturnType();
    boolean isAbstract();
    boolean isDefaultMethod();
    boolean isStatic();
}
