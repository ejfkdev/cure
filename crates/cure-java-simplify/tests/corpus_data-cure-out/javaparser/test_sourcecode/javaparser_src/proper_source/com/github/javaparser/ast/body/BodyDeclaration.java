package com.github.javaparser.ast.body;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import java.util.ArrayList;
import java.util.List;

public abstract class BodyDeclaration extends Node implements AnnotableNode {
    private List<AnnotationExpr> annotations;
    public BodyDeclaration() {}
    public BodyDeclaration(List<AnnotationExpr> annotations) {
        setAnnotations(annotations);
    }
    public BodyDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, List<AnnotationExpr> annotations) {
        super(beginLine, beginColumn, endLine, endColumn);
        setAnnotations(annotations);
    }
    public final List<AnnotationExpr> getAnnotations() {
        if (annotations == null) {
            annotations = new ArrayList<AnnotationExpr>();
        }
        return annotations;
    }
    public final void setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
        setAsParentNodeOf(this.annotations);
    }
}
