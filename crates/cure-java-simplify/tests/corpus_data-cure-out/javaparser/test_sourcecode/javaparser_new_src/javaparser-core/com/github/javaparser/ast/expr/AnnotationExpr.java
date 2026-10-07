package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;

public abstract class AnnotationExpr extends Expression {
    protected NameExpr name;
    public AnnotationExpr() {}
    public AnnotationExpr(Range range) {
        super(range);
    }
    public NameExpr getName() {
        return name;
    }
    public AnnotationExpr setName(NameExpr name) {
        this.name = name;
        setAsParentNodeOf(name);
        return this;
    }
}
