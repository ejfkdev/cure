package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class BreakStmt extends Statement {
    private String id;
    public BreakStmt() {}
    public BreakStmt(final String id) {
        this.id = id;
    }
    public BreakStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String id) {
        super(beginLine, beginColumn, endLine, endColumn);
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
    public void setId(final String id) {
        this.id = id;
    }
}
