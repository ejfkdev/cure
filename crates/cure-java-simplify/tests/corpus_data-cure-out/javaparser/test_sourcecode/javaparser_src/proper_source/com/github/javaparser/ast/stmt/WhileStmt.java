package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class WhileStmt extends Statement {
    private Expression condition;
    private Statement body;
    public WhileStmt() {}
    public WhileStmt(final Expression condition, final Statement body) {
        setCondition(condition);
        setBody(body);
    }
    public WhileStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression condition, final Statement body) {
        super(beginLine, beginColumn, endLine, endColumn);
        setCondition(condition);
        setBody(body);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Statement getBody() {
        return body;
    }
    public Expression getCondition() {
        return condition;
    }
    public void setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
    }
    public void setCondition(final Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
    }
}
