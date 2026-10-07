package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.utils.Utils;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class StringLiteralExpr extends LiteralExpr {
    protected String value;
    public StringLiteralExpr() {
        this.value = "";
    }
    public StringLiteralExpr(final String value) {
        if (value.contains("\n") || value.contains("\r")) {
            throw new IllegalArgumentException("Illegal literal expression: newlines (line feed or carriage return) have to be escaped");
        }
        this.value = value;
    }
    public static StringLiteralExpr escape(String string) {
        return new StringLiteralExpr(Utils.escapeEndOfLines(string));
    }
    public StringLiteralExpr(final Range range, final String value) {
        super(range);
        this.value = value;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public final String getValue() {
        return value;
    }
    public final StringLiteralExpr setValue(final String value) {
        this.value = value;
        return this;
    }
}
