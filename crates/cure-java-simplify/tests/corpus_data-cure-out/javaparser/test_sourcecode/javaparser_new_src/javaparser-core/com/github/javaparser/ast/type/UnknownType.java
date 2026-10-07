package com.github.javaparser.ast.type;

import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class UnknownType extends Type<UnknownType> {
    public UnknownType() {}
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    @Override
    public List<AnnotationExpr> getAnnotations() {
        throw new IllegalStateException("Inferred lambda types cannot be annotated.");
    }
    @Override
    public UnknownType setAnnotations(List<AnnotationExpr> annotations) {
        throw new IllegalStateException("Inferred lambda types cannot be annotated.");
    }
}
