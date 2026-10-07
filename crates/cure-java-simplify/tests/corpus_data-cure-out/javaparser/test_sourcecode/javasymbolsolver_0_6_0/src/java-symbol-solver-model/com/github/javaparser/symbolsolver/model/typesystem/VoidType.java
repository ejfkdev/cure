package com.github.javaparser.symbolsolver.model.typesystem;

public class VoidType implements Type {
    public static final Type INSTANCE = new VoidType();
    private VoidType() {}
    @Override
    public String describe() {
        return "void";
    }
    @Override
    public boolean isAssignableBy(Type other) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isVoid() {
        return true;
    }
}
