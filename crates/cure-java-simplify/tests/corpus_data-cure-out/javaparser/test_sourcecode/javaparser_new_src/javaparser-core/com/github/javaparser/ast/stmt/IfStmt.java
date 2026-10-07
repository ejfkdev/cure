package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class IfStmt extends Statement {
    private Expression condition;
    private Statement thenStmt;
    private Statement elseStmt;
    public IfStmt() {}
    public IfStmt(final Expression condition, final Statement thenStmt, final Statement elseStmt) {
        setCondition(condition);
        setThenStmt(thenStmt);
        setElseStmt(elseStmt);
    }
    public IfStmt(Range range, final Expression condition, final Statement thenStmt, final Statement elseStmt) {
        super(range);
        setCondition(condition);
        setThenStmt(thenStmt);
        setElseStmt(elseStmt);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getCondition() {
        return condition;
    }
    public Statement getElseStmt() {
        return elseStmt;
    }
    public Statement getThenStmt() {
        return thenStmt;
    }
    public IfStmt setCondition(final Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
        return this;
    }
    public IfStmt setElseStmt(final Statement elseStmt) {
        this.elseStmt = elseStmt;
        setAsParentNodeOf(this.elseStmt);
        return this;
    }
    public IfStmt setThenStmt(final Statement thenStmt) {
        this.thenStmt = thenStmt;
        setAsParentNodeOf(this.thenStmt);
        return this;
    }
}
