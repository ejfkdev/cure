package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class AssignExpr extends Expression {
    public enum Operator {
        assign, plus, minus, star, slash, and, or, xor, rem, lShift, rSignedShift, rUnsignedShift
    }
    private Expression target;
    private Expression value;
    private Operator op;
    public AssignExpr() {}
    public AssignExpr(Expression target, Expression value, Operator op) {
        setTarget(target);
        setValue(value);
        setOperator(op);
    }
    public AssignExpr(Range range, Expression target, Expression value, Operator op) {
        super(range);
        setTarget(target);
        setValue(value);
        setOperator(op);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Operator getOperator() {
        return op;
    }
    public Expression getTarget() {
        return target;
    }
    public Expression getValue() {
        return value;
    }
    public AssignExpr setOperator(Operator op) {
        this.op = op;
        return this;
    }
    public AssignExpr setTarget(Expression target) {
        this.target = target;
        setAsParentNodeOf(this.target);
        return this;
    }
    public AssignExpr setValue(Expression value) {
        this.value = value;
        setAsParentNodeOf(this.value);
        return this;
    }
}
