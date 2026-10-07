package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import java.util.List;
import static com.github.javaparser.utils.Utils.*;

public abstract class Type<T extends Type> extends Node {
    private List<AnnotationExpr> annotations;
    public Type() {}
    public Type(List<AnnotationExpr> annotation) {
        this.annotations = annotation;
    }
    public Type(Range range) {
        super(range);
    }
    public Type(Range range, List<AnnotationExpr> annotations) {
        super(range);
        setAnnotations(annotations);
    }
    public List<AnnotationExpr> getAnnotations() {
        annotations = ensureNotNull(annotations);
        return annotations;
    }
    public T setAnnotations(List<AnnotationExpr> annotations) {
        setAsParentNodeOf(annotations);
        this.annotations = annotations;
        return (T) this;
    }
}
