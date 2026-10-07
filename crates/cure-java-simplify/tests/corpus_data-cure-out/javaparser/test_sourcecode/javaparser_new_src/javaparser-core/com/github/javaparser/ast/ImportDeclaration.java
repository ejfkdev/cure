package com.github.javaparser.ast;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ImportDeclaration extends Node {
    private NameExpr name;
    private boolean static_;
    private boolean asterisk;
    private boolean isEmptyImportDeclaration;
    private ImportDeclaration() {
        this.isEmptyImportDeclaration = true;
        static_ = false;
        asterisk = false;
    }
    private ImportDeclaration(Range range) {
        super(range);
        this.isEmptyImportDeclaration = true;
        static_ = false;
        asterisk = false;
    }
    public static ImportDeclaration createEmptyDeclaration() {
        return new ImportDeclaration();
    }
    public static ImportDeclaration createEmptyDeclaration(Range range) {
        return new ImportDeclaration(range);
    }
    public ImportDeclaration(NameExpr name, boolean isStatic, boolean isAsterisk) {
        setAsterisk(isAsterisk);
        setName(name);
        setStatic(isStatic);
        this.isEmptyImportDeclaration = false;
    }
    public ImportDeclaration(Range range, NameExpr name, boolean isStatic, boolean isAsterisk) {
        super(range);
        setAsterisk(isAsterisk);
        setName(name);
        setStatic(isStatic);
        this.isEmptyImportDeclaration = false;
    }
    public boolean isEmptyImportDeclaration() {
        return this.isEmptyImportDeclaration;
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public NameExpr getName() {
        if (isEmptyImportDeclaration) {
            throw new UnsupportedOperationException("Empty import declarations have no name");
        }
        return name;
    }
    public boolean isAsterisk() {
        return asterisk;
    }
    public boolean isStatic() {
        return static_;
    }
    public ImportDeclaration setAsterisk(boolean asterisk) {
        if (isEmptyImportDeclaration && asterisk) {
            throw new UnsupportedOperationException("Empty import cannot have asterisk");
        }
        this.asterisk = asterisk;
        return this;
    }
    public ImportDeclaration setName(NameExpr name) {
        if (isEmptyImportDeclaration) {
            throw new UnsupportedOperationException("Empty import cannot have name");
        }
        this.name = name;
        setAsParentNodeOf(this.name);
        return this;
    }
    public ImportDeclaration setStatic(boolean static_) {
        if (isEmptyImportDeclaration && static_) {
            throw new UnsupportedOperationException("Empty import cannot be static");
        }
        this.static_ = static_;
        return this;
    }
}
