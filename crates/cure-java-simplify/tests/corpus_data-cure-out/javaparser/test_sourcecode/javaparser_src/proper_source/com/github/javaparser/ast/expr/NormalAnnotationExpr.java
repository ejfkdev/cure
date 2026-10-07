package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class NormalAnnotationExpr extends AnnotationExpr {
    private List<MemberValuePair> pairs;
    public NormalAnnotationExpr() {}
    public NormalAnnotationExpr(final NameExpr name, final List<MemberValuePair> pairs) {
        setName(name);
        setPairs(pairs);
    }
    public NormalAnnotationExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final NameExpr name, final List<MemberValuePair> pairs) {
        super(beginLine, beginColumn, endLine, endColumn);
        setName(name);
        setPairs(pairs);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<MemberValuePair> getPairs() {
        return pairs;
    }
    public void setPairs(final List<MemberValuePair> pairs) {
        this.pairs = pairs;
        setAsParentNodeOf(this.pairs);
    }
}
