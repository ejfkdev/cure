package com.github.javaparser.ast.expr;

public abstract class AnnotationExpr extends Expression {
    protected NameExpr name;
    public AnnotationExpr() {}
    public AnnotationExpr(int beginLine, int beginColumn, int endLine, int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn);
    }
    public NameExpr getName() {
        return name;
    }
    public void setName(NameExpr name) {
        this.name = name;
        setAsParentNodeOf(name);
    }
}
