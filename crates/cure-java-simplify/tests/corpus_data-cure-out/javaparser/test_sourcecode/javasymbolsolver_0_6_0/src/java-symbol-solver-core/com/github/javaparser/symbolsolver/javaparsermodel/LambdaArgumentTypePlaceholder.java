package com.github.javaparser.symbolsolver.javaparsermodel;

import com.github.javaparser.symbolsolver.model.declarations.MethodLikeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.typesystem.Type;

public class LambdaArgumentTypePlaceholder implements Type {
    private int pos;
    private SymbolReference<? extends MethodLikeDeclaration> method;
    public LambdaArgumentTypePlaceholder(int pos) {
        this.pos = pos;
    }
    @Override
    public boolean isArray() {
        return false;
    }
    @Override
    public boolean isPrimitive() {
        return false;
    }
    @Override
    public boolean isReferenceType() {
        return false;
    }
    @Override
    public String describe() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isTypeVariable() {
        return false;
    }
    public void setMethod(SymbolReference<? extends MethodLikeDeclaration> method) {
        this.method = method;
    }
    @Override
    public boolean isAssignableBy(Type other) {
        throw new UnsupportedOperationException();
    }
}
