package com.github.javaparser.symbolsolver.model.typesystem;

@FunctionalInterface
public interface TypeTransformer {
    Type transform(Type type);
}
