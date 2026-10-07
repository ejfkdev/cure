package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ClassExpr extends Expression implements NodeWithType<ClassExpr> {
    private Type type;
    public ClassExpr() {}
    public ClassExpr(Type type) {
        setType(type);
    }
    public ClassExpr(Range range, Type type) {
        super(range);
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
    @Override
    public Type getType() {
        return type;
    }
    @Override
    public ClassExpr setType(Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
        return this;
    }
}
