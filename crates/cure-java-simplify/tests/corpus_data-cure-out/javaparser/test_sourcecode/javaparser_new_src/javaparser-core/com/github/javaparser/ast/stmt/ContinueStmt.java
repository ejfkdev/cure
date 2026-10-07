package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ContinueStmt extends Statement {
    private String id;
    public ContinueStmt() {}
    public ContinueStmt(final String id) {
        this.id = id;
    }
    public ContinueStmt(Range range, final String id) {
        super(range);
        this.id = id;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public String getId() {
        return id;
    }
    public ContinueStmt setId(final String id) {
        this.id = id;
        return this;
    }
}
