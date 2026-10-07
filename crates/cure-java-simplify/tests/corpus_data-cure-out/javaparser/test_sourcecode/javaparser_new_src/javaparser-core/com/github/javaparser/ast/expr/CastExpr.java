package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class CastExpr extends Expression implements NodeWithType<CastExpr> {
    private Type type;
    private Expression expr;
    public CastExpr() {}
    public CastExpr(Type type, Expression expr) {
        setType(type);
        setExpr(expr);
    }
    public CastExpr(Range range, Type type, Expression expr) {
        super(range);
        setType(type);
        setExpr(expr);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Expression getExpr() {
        return expr;
    }
    @Override
    public Type getType() {
        return type;
    }
    public CastExpr setExpr(Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
        return this;
    }
    @Override
    public CastExpr setType(Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
        return this;
    }
}
