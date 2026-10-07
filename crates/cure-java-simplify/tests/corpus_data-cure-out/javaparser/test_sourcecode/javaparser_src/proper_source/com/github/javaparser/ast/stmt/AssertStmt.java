package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class AssertStmt extends Statement {
    private Expression check;
    private Expression msg;
    public AssertStmt() {}
    public AssertStmt(final Expression check) {
        setCheck(check);
    }
    public AssertStmt(final Expression check, final Expression msg) {
        setCheck(check);
        setMessage(msg);
    }
    public AssertStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression check, final Expression msg) {
        super(beginLine, beginColumn, endLine, endColumn);
        setCheck(check);
        setMessage(msg);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getCheck() {
        return check;
    }
    public Expression getMessage() {
        return msg;
    }
    public void setCheck(final Expression check) {
        this.check = check;
        setAsParentNodeOf(this.check);
    }
    public void setMessage(final Expression msg) {
        this.msg = msg;
        setAsParentNodeOf(this.msg);
    }
}
