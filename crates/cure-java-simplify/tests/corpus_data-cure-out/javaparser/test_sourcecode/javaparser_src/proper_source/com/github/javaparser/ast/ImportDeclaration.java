package com.github.javaparser.ast;

import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ImportDeclaration extends Node {
    private NameExpr name;
    private boolean static_;
    private boolean asterisk;
    public ImportDeclaration() {}
    public ImportDeclaration(NameExpr name, boolean isStatic, boolean isAsterisk) {
        setAsterisk(isAsterisk);
        setName(name);
        setStatic(isStatic);
    }
    public ImportDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, NameExpr name, boolean isStatic, boolean isAsterisk) {
        super(beginLine, beginColumn, endLine, endColumn);
        setAsterisk(isAsterisk);
        setName(name);
        setStatic(isStatic);
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
        return name;
    }
    public boolean isAsterisk() {
        return asterisk;
    }
    public boolean isStatic() {
        return static_;
    }
    public void setAsterisk(boolean asterisk) {
        this.asterisk = asterisk;
    }
    public void setName(NameExpr name) {
        this.name = name;
        setAsParentNodeOf(this.name);
    }
    public void setStatic(boolean static_) {
        this.static_ = static_;
    }
}
