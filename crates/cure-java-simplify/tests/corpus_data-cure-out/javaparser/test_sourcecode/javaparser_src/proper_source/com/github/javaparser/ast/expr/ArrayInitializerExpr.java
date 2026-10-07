package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ArrayInitializerExpr extends Expression {
    private List<Expression> values;
    public ArrayInitializerExpr() {}
    public ArrayInitializerExpr(List<Expression> values) {
        setValues(values);
    }
    public ArrayInitializerExpr(int beginLine, int beginColumn, int endLine, int endColumn, List<Expression> values) {
        super(beginLine, beginColumn, endLine, endColumn);
        setValues(values);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public List<Expression> getValues() {
        return values;
    }
    public void setValues(List<Expression> values) {
        this.values = values;
        setAsParentNodeOf(this.values);
    }
}
