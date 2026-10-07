package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class BinaryExpr extends Expression {
    public enum Operator {
        or, and, binOr, binAnd, xor, equals, notEquals, less, greater, lessEquals, greaterEquals, lShift, rSignedShift, rUnsignedShift, plus, minus, times, divide, remainder
    }
    private Expression left;
    private Expression right;
    private Operator op;
    public BinaryExpr() {}
    public BinaryExpr(Expression left, Expression right, Operator op) {
        setLeft(left);
        setRight(right);
        setOperator(op);
    }
    public BinaryExpr(Range range, Expression left, Expression right, Operator op) {
        super(range);
        setLeft(left);
        setRight(right);
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
    public Expression getLeft() {
        return left;
    }
    public Operator getOperator() {
        return op;
    }
    public Expression getRight() {
        return right;
    }
    public BinaryExpr setLeft(Expression left) {
        this.left = left;
        setAsParentNodeOf(this.left);
        return this;
    }
    public BinaryExpr setOperator(Operator op) {
        this.op = op;
        return this;
    }
    public BinaryExpr setRight(Expression right) {
        this.right = right;
        setAsParentNodeOf(this.right);
        return this;
    }
}
