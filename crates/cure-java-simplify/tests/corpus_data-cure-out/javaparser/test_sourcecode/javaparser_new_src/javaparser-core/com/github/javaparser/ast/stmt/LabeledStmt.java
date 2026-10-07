package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class LabeledStmt extends Statement {
    private String label;
    private Statement stmt;
    public LabeledStmt() {}
    public LabeledStmt(final String label, final Statement stmt) {
        setLabel(label);
        setStmt(stmt);
    }
    public LabeledStmt(Range range, final String label, final Statement stmt) {
        super(range);
        setLabel(label);
        setStmt(stmt);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public String getLabel() {
        return label;
    }
    public Statement getStmt() {
        return stmt;
    }
    public LabeledStmt setLabel(final String label) {
        this.label = label;
        return this;
    }
    public LabeledStmt setStmt(final Statement stmt) {
        this.stmt = stmt;
        setAsParentNodeOf(this.stmt);
        return this;
    }
}
