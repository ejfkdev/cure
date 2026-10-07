package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class InstanceOfExpr extends Expression {
    private Expression expr;
    private Type type;
    public InstanceOfExpr() {}
    public InstanceOfExpr(final Expression expr, final Type type) {
        setExpr(expr);
        setType(type);
    }
    public InstanceOfExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression expr, final Type type) {
        super(beginLine, beginColumn, endLine, endColumn);
        setExpr(expr);
        setType(type);
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
    public Type getType() {
        return type;
    }
    public void setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
    public void setType(final Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
}
