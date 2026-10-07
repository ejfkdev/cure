package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class MarkerAnnotationExpr extends AnnotationExpr {
    public MarkerAnnotationExpr() {}
    public MarkerAnnotationExpr(final NameExpr name) {
        setName(name);
    }
    public MarkerAnnotationExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final NameExpr name) {
        super(beginLine, beginColumn, endLine, endColumn);
        setName(name);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
}
