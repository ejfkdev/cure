package com.github.javaparser.ast.body;

import com.github.javaparser.ast.expr.AnnotationExpr;
import java.util.List;

public interface AnnotableNode {
    public List<AnnotationExpr> getAnnotations();
}
