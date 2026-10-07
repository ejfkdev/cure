package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ThisExpr extends Expression {
    private Expression classExpr;
    public ThisExpr() {}
    public ThisExpr(final Expression classExpr) {
        setClassExpr(classExpr);
    }
    public ThisExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression classExpr) {
        super(beginLine, beginColumn, endLine, endColumn);
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
    public void setClassExpr(final Expression classExpr) {
        this.classExpr = classExpr;
        setAsParentNodeOf(this.classExpr);
    }
}
