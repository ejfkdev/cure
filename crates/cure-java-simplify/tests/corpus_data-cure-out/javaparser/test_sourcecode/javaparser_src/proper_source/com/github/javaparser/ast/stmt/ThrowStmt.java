package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ThrowStmt extends Statement {
    private Expression expr;
    public ThrowStmt() {}
    public ThrowStmt(final Expression expr) {
        setExpr(expr);
    }
    public ThrowStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression expr) {
        super(beginLine, beginColumn, endLine, endColumn);
        setExpr(expr);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getExpr() {
        return expr;
    }
    public void setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
}
