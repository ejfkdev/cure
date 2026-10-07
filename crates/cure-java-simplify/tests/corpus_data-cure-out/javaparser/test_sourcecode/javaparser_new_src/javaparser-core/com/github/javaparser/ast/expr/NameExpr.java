package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public class NameExpr extends Expression implements NodeWithName<NameExpr> {
    private String name;
    public NameExpr() {}
    public NameExpr(final String name) {
        this.name = name;
    }
    public NameExpr(Range range, final String name) {
        super(range);
        this.name = name;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    @Override
	public final String getName() {
        return name;
    }
    @Override
    public NameExpr setName(final String name) {
        this.name = name;
        return this;
    }
    public static NameExpr name(String qualifiedName) {
        String[] split = qualifiedName.split("\\.");
        NameExpr ret = new NameExpr(split[0]);
        for (int i = 1; i < split.length; i++) {
            ret = new QualifiedNameExpr(ret, split[i]);
        }
        return ret;
    }
}
