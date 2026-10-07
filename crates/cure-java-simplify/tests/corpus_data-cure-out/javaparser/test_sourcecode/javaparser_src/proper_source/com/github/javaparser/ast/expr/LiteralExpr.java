package com.github.javaparser.ast.expr;

public abstract class LiteralExpr extends Expression {
    public LiteralExpr() {}
    public LiteralExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn);
    }
}
