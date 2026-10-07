package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ThisExpr extends Expression {
    private Expression classExpr;
    public ThisExpr() {}
    public ThisExpr(final Expression classExpr) {
        setClassExpr(classExpr);
    }
    public ThisExpr(final Range range, final Expression classExpr) {
        super(range);
        setClassExpr(classExpr);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getClassExpr() {
        return classExpr;
    }
    public ThisExpr setClassExpr(final Expression classExpr) {
        this.classExpr = classExpr;
        setAsParentNodeOf(this.classExpr);
        return this;
    }
}
