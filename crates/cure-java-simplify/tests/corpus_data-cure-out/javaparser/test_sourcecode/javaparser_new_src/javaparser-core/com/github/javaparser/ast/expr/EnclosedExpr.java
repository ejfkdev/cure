package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class EnclosedExpr extends Expression {
    private Expression inner;
    public EnclosedExpr() {}
    public EnclosedExpr(final Expression inner) {
        setInner(inner);
    }
    public EnclosedExpr(final Range range, final Expression inner) {
        super(range);
        setInner(inner);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getInner() {
        return inner;
    }
    public EnclosedExpr setInner(final Expression inner) {
        this.inner = inner;
        setAsParentNodeOf(this.inner);
        return this;
    }
}
