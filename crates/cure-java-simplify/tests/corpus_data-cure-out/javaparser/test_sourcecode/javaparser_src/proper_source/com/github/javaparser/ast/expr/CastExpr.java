package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class CastExpr extends Expression {
    private Type type;
    private Expression expr;
    public CastExpr() {}
    public CastExpr(Type type, Expression expr) {
        setType(type);
        setExpr(expr);
    }
    public CastExpr(int beginLine, int beginColumn, int endLine, int endColumn, Type type, Expression expr) {
        super(beginLine, beginColumn, endLine, endColumn);
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
    public Type getType() {
        return type;
    }
    public void setExpr(Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
    public void setType(Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
}
