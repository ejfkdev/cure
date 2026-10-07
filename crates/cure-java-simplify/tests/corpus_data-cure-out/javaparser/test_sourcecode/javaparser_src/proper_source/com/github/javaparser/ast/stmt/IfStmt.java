package com.github.javaparser.ast.stmt;

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
    public IfStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression condition, final Statement thenStmt, final Statement elseStmt) {
        super(beginLine, beginColumn, endLine, endColumn);
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
    public void setCondition(final Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
    }
    public void setElseStmt(final Statement elseStmt) {
        this.elseStmt = elseStmt;
        setAsParentNodeOf(this.elseStmt);
    }
    public void setThenStmt(final Statement thenStmt) {
        this.thenStmt = thenStmt;
        setAsParentNodeOf(this.thenStmt);
    }
}
