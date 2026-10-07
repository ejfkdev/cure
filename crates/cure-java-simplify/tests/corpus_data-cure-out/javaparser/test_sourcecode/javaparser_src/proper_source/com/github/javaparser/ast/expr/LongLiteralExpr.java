package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class LongLiteralExpr extends StringLiteralExpr {
    private static final String UNSIGNED_MIN_VALUE = "9223372036854775808";
    protected static final String MIN_VALUE = "-" + UNSIGNED_MIN_VALUE + "L";
    public LongLiteralExpr() {}
    public LongLiteralExpr(final String value) {
        super(value);
    }
    public LongLiteralExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String value) {
        super(beginLine, beginColumn, endLine, endColumn, value);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public final boolean isMinValue() {
        return value != null && value.length() == 20 && value.startsWith(UNSIGNED_MIN_VALUE) && (value.charAt(19) == 'L' || value.charAt(19) == 'l');
    }
}
