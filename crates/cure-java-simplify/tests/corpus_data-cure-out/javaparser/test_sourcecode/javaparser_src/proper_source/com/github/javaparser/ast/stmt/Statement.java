package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.Node;

public abstract class Statement extends Node {
    public Statement() {}
    public Statement(final int beginLine, final int beginColumn, final int endLine, final int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn);
    }
}
