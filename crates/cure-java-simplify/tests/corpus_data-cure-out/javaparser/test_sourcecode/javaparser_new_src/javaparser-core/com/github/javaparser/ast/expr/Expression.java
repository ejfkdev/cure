package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;

public abstract class Expression extends Node {
    public Expression() {}
    public Expression(Range range) {
        super(range);
    }
}
