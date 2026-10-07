package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;

public abstract class Statement extends Node {
    public Statement() {}
    public Statement(final Range range) {
        super(range);
    }
}
