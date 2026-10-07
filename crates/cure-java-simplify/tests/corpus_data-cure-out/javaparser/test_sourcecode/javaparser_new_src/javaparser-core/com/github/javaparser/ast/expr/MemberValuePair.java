package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class MemberValuePair extends Node implements NodeWithName<MemberValuePair> {
    private String name;
    private Expression value;
    public MemberValuePair() {}
    public MemberValuePair(final String name, final Expression value) {
        setName(name);
        setValue(value);
    }
    public MemberValuePair(final Range range, final String name, final Expression value) {
        super(range);
        setName(name);
        setValue(value);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    @Override
	public String getName() {
        return name;
    }
    public Expression getValue() {
        return value;
    }
    @Override
    public MemberValuePair setName(final String name) {
        this.name = name;
        return this;
    }
    public MemberValuePair setValue(final Expression value) {
        this.value = value;
        setAsParentNodeOf(this.value);
        return this;
    }
}
