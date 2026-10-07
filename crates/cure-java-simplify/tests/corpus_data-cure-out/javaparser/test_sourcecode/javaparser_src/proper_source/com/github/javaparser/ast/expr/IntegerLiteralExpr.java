package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class IntegerLiteralExpr extends StringLiteralExpr {
    private static final String UNSIGNED_MIN_VALUE = "2147483648";
    protected static final String MIN_VALUE = "-" + UNSIGNED_MIN_VALUE;
    public IntegerLiteralExpr() {}
    public IntegerLiteralExpr(final String value) {
        super(value);
    }
    public IntegerLiteralExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String value) {
        super(beginLine, beginColumn, endLine, endColumn, value);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public final boolean isMinValue() {
        return value != null && value.length() == 10 && value.equals(UNSIGNED_MIN_VALUE);
    }
}
