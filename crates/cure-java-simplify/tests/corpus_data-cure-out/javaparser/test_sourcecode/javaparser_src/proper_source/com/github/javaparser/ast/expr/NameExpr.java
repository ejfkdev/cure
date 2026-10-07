package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.NamedNode;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class NameExpr extends Expression implements NamedNode {
    private String name;
    public NameExpr() {}
    public NameExpr(final String name) {
        this.name = name;
    }
    public NameExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String name) {
        super(beginLine, beginColumn, endLine, endColumn);
        this.name = name;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public final String getName() {
        return name;
    }
    public final void setName(final String name) {
        this.name = name;
    }
}
