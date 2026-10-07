package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ConditionalExpr extends Expression {
    private Expression condition;
    private Expression thenExpr;
    private Expression elseExpr;
    public ConditionalExpr() {}
    public ConditionalExpr(Expression condition, Expression thenExpr, Expression elseExpr) {
        setCondition(condition);
        setThenExpr(thenExpr);
        setElseExpr(elseExpr);
    }
    public ConditionalExpr(Range range, Expression condition, Expression thenExpr, Expression elseExpr) {
        super(range);
        setCondition(condition);
        setThenExpr(thenExpr);
        setElseExpr(elseExpr);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Expression getCondition() {
        return condition;
    }
    public Expression getElseExpr() {
        return elseExpr;
    }
    public Expression getThenExpr() {
        return thenExpr;
    }
    public ConditionalExpr setCondition(Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
        return this;
    }
    public ConditionalExpr setElseExpr(Expression elseExpr) {
        this.elseExpr = elseExpr;
        setAsParentNodeOf(this.elseExpr);
        return this;
    }
    public ConditionalExpr setThenExpr(Expression thenExpr) {
        this.thenExpr = thenExpr;
        setAsParentNodeOf(this.thenExpr);
        return this;
    }
}
