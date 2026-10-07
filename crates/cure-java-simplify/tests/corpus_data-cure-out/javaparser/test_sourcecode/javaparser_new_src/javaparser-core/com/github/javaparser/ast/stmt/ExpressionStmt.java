package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ExpressionStmt extends Statement {
    private Expression expr;
    public ExpressionStmt() {}
    public ExpressionStmt(final Expression expr) {
        setExpression(expr);
    }
    public ExpressionStmt(Range range, final Expression expr) {
        super(range);
        setExpression(expr);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getExpression() {
        return expr;
    }
    public ExpressionStmt setExpression(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
        return this;
    }
}
