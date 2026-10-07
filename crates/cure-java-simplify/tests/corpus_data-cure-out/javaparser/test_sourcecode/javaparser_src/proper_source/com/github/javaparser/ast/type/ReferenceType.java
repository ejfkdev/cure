package com.github.javaparser.ast.type;

import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ReferenceType extends Type {
    private Type type;
    private int arrayCount;
    private List<List<AnnotationExpr>> arraysAnnotations;
    public ReferenceType() {}
    public ReferenceType(final Type type) {
        setType(type);
    }
    public ReferenceType(final Type type, final int arrayCount) {
        setType(type);
        setArrayCount(arrayCount);
    }
    public ReferenceType(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Type type, final int arrayCount) {
        super(beginLine, beginColumn, endLine, endColumn);
        setType(type);
        setArrayCount(arrayCount);
    }
    public ReferenceType(int beginLine, int beginColumn, int endLine, int endColumn, Type type, int arrayCount, List<AnnotationExpr> annotations, List<List<AnnotationExpr>> arraysAnnotations) {
        super(beginLine, beginColumn, endLine, endColumn, annotations);
        setType(type);
        setArrayCount(arrayCount);
        this.arraysAnnotations = arraysAnnotations;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public int getArrayCount() {
        return arrayCount;
    }
    public Type getType() {
        return type;
    }
    public void setArrayCount(final int arrayCount) {
        this.arrayCount = arrayCount;
    }
    public void setType(final Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
    public List<List<AnnotationExpr>> getArraysAnnotations() {
        return arraysAnnotations;
    }
    public void setArraysAnnotations(List<List<AnnotationExpr>> arraysAnnotations) {
        this.arraysAnnotations = arraysAnnotations;
    }
}
