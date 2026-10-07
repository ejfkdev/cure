package com.github.javaparser.ast.body;

import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.utils.Utils;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;

public abstract class BodyDeclaration<T> extends Node implements NodeWithAnnotations<T> {
    private List<AnnotationExpr> annotations;
    public BodyDeclaration() {}
    public BodyDeclaration(List<AnnotationExpr> annotations) {
        setAnnotations(annotations);
    }
    public BodyDeclaration(Range range, List<AnnotationExpr> annotations) {
        super(range);
        setAnnotations(annotations);
    }
    @Override
    public final List<AnnotationExpr> getAnnotations() {
        annotations = Utils.ensureNotNull(annotations);
        return annotations;
    }
    @SuppressWarnings("unchecked")
    @Override
    public final T setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
        setAsParentNodeOf(this.annotations);
        return (T) this;
    }
}
