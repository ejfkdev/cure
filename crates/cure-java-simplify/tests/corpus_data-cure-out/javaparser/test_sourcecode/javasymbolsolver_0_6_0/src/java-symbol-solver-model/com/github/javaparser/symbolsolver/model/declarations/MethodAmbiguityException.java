package com.github.javaparser.symbolsolver.model.declarations;

public class MethodAmbiguityException extends RuntimeException {
    public MethodAmbiguityException(String description) {
        super(description);
    }
}
