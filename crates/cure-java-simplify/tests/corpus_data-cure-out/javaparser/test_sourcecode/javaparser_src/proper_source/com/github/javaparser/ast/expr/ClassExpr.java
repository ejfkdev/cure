package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ClassExpr extends Expression {
    private Type type;
    public ClassExpr() {}
    public ClassExpr(Type type) {
        setType(type);
    }
    public ClassExpr(int beginLine, int beginColumn, int endLine, int endColumn, Type type) {
        super(beginLine, beginColumn, endLine, endColumn);
        setType(type);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Type getType() {
        return type;
    }
    public void setType(Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
}
