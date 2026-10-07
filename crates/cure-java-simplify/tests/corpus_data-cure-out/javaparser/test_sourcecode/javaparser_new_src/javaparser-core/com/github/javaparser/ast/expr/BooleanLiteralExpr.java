package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class BooleanLiteralExpr extends LiteralExpr {
    private boolean value;
    public BooleanLiteralExpr() {}
    public BooleanLiteralExpr(boolean value) {
        setValue(value);
    }
    public BooleanLiteralExpr(Range range, boolean value) {
        super(range);
        setValue(value);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public boolean getValue() {
        return value;
    }
    public BooleanLiteralExpr setValue(boolean value) {
        this.value = value;
        return this;
    }
}
