package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;

public abstract class LiteralExpr extends Expression {
    public LiteralExpr() {}
    public LiteralExpr(Range range) {
        super(range);
    }
}
