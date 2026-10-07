package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ArrayAccessExpr extends Expression {
    private Expression name;
    private Expression index;
    public ArrayAccessExpr() {}
    public ArrayAccessExpr(Expression name, Expression index) {
        setName(name);
        setIndex(index);
    }
    public ArrayAccessExpr(Range range, Expression name, Expression index) {
        super(range);
        setName(name);
        setIndex(index);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Expression getIndex() {
        return index;
    }
    public Expression getName() {
        return name;
    }
    public ArrayAccessExpr setIndex(Expression index) {
        this.index = index;
        setAsParentNodeOf(this.index);
        return this;
    }
    public ArrayAccessExpr setName(Expression name) {
        this.name = name;
        setAsParentNodeOf(this.name);
        return this;
    }
}
