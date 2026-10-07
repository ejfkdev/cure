package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class UnaryExpr extends Expression {
    public static enum Operator {
        positive, negative, preIncrement, preDecrement, not, inverse, posIncrement, posDecrement
    }
    private Expression expr;
    private Operator op;
    public UnaryExpr() {}
    public UnaryExpr(final Expression expr, final Operator op) {
        setExpr(expr);
        setOperator(op);
    }
    public UnaryExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression expr, final Operator op) {
        super(beginLine, beginColumn, endLine, endColumn);
        setExpr(expr);
        setOperator(op);
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
    public Operator getOperator() {
        return op;
    }
    public void setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
    public void setOperator(final Operator op) {
        this.op = op;
    }
}
