package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class LongLiteralMinValueExpr extends LongLiteralExpr {
    public LongLiteralMinValueExpr() {
        super(MIN_VALUE);
    }
    public LongLiteralMinValueExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn, MIN_VALUE);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
}
