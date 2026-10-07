package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class StringLiteralExpr extends LiteralExpr {
    protected String value;
    public StringLiteralExpr() {}
    public StringLiteralExpr(final String value) {
        this.value = value;
    }
    public StringLiteralExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String value) {
        super(beginLine, beginColumn, endLine, endColumn);
        this.value = value;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public final String getValue() {
        return value;
    }
    public final void setValue(final String value) {
        this.value = value;
    }
}
