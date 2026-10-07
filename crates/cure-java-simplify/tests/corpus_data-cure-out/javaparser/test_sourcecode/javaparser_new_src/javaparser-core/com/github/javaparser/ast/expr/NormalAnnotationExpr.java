package com.github.javaparser.ast.expr;

import static com.github.javaparser.ast.expr.NameExpr.*;
import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class NormalAnnotationExpr extends AnnotationExpr {
    private List<MemberValuePair> pairs;
    public NormalAnnotationExpr() {}
    public NormalAnnotationExpr(final NameExpr name, final List<MemberValuePair> pairs) {
        setName(name);
        setPairs(pairs);
    }
    public NormalAnnotationExpr(final Range range, final NameExpr name, final List<MemberValuePair> pairs) {
        super(range);
        setName(name);
        setPairs(pairs);
    }
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<MemberValuePair> getPairs() {
        pairs = ensureNotNull(pairs);
        return pairs;
    }
    public NormalAnnotationExpr setPairs(final List<MemberValuePair> pairs) {
        this.pairs = pairs;
        setAsParentNodeOf(this.pairs);
        return this;
    }
    public NormalAnnotationExpr addPair(String key, String value) {
        return addPair(key, name(value));
    }
    public NormalAnnotationExpr addPair(String key, NameExpr value) {
        MemberValuePair memberValuePair = new MemberValuePair(key, value);
        getPairs().add(memberValuePair);
        memberValuePair.setParentNode(this);
        return this;
    }
}
