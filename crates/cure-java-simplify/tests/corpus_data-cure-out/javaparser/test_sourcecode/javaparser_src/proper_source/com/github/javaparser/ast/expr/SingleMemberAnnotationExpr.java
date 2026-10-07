package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class SingleMemberAnnotationExpr extends AnnotationExpr {
    private Expression memberValue;
    public SingleMemberAnnotationExpr() {}
    public SingleMemberAnnotationExpr(final NameExpr name, final Expression memberValue) {
        setName(name);
        setMemberValue(memberValue);
    }
    public SingleMemberAnnotationExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final NameExpr name, final Expression memberValue) {
        super(beginLine, beginColumn, endLine, endColumn);
        setName(name);
        setMemberValue(memberValue);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getMemberValue() {
        return memberValue;
    }
    public void setMemberValue(final Expression memberValue) {
        this.memberValue = memberValue;
        setAsParentNodeOf(this.memberValue);
    }
}
