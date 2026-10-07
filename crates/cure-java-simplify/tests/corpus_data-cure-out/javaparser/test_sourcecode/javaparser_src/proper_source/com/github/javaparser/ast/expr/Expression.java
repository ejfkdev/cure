package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.Node;

public abstract class Expression extends Node {
    public Expression() {}
    public Expression(final int beginLine, final int beginColumn, final int endLine, final int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn);
    }
}
