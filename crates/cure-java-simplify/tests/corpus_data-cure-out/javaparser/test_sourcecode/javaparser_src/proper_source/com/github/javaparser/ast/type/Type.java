package com.github.javaparser.ast.type;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import java.util.List;

public abstract class Type extends Node {
    private List<AnnotationExpr> annotations;
    public Type() {}
    public Type(List<AnnotationExpr> annotation) {
        this.annotations = annotation;
    }
    public Type(int beginLine, int beginColumn, int endLine, int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn);
    }
    public Type(int beginLine, int beginColumn, int endLine, int endColumn, List<AnnotationExpr> annotations) {
        super(beginLine, beginColumn, endLine, endColumn);
        this.annotations = annotations;
    }
    public List<AnnotationExpr> getAnnotations() {
        return annotations;
    }
    public void setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
    }
}
