package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class QualifiedNameExpr extends NameExpr {
    private NameExpr qualifier;
    public QualifiedNameExpr() {}
    public QualifiedNameExpr(final NameExpr scope, final String name) {
        super(name);
        setQualifier(scope);
    }
    public QualifiedNameExpr(final Range range, final NameExpr scope, final String name) {
        super(range, name);
        setQualifier(scope);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public NameExpr getQualifier() {
        return qualifier;
    }
    public QualifiedNameExpr setQualifier(final NameExpr qualifier) {
        this.qualifier = qualifier;
        setAsParentNodeOf(this.qualifier);
        return this;
    }
}
