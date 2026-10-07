package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithBody;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class DoStmt extends Statement implements NodeWithBody<DoStmt> {
    private Statement body;
    private Expression condition;
    public DoStmt() {}
    public DoStmt(final Statement body, final Expression condition) {
        setBody(body);
        setCondition(condition);
    }
    public DoStmt(Range range, final Statement body, final Expression condition) {
        super(range);
        setBody(body);
        setCondition(condition);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    @Override
    public Statement getBody() {
        return body;
    }
    public Expression getCondition() {
        return condition;
    }
    @Override
    public DoStmt setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
        return this;
    }
    public DoStmt setCondition(final Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
        return this;
    }
}
