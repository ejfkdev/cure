package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ForeachStmt extends Statement {
    private VariableDeclarationExpr var;
    private Expression iterable;
    private Statement body;
    public ForeachStmt() {}
    public ForeachStmt(final VariableDeclarationExpr var, final Expression iterable, final Statement body) {
        setVariable(var);
        setIterable(iterable);
        setBody(body);
    }
    public ForeachStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final VariableDeclarationExpr var, final Expression iterable, final Statement body) {
        super(beginLine, beginColumn, endLine, endColumn);
        setVariable(var);
        setIterable(iterable);
        setBody(body);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Statement getBody() {
        return body;
    }
    public Expression getIterable() {
        return iterable;
    }
    public VariableDeclarationExpr getVariable() {
        return var;
    }
    public void setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
    }
    public void setIterable(final Expression iterable) {
        this.iterable = iterable;
        setAsParentNodeOf(this.iterable);
    }
    public void setVariable(final VariableDeclarationExpr var) {
        this.var = var;
        setAsParentNodeOf(this.var);
    }
}
