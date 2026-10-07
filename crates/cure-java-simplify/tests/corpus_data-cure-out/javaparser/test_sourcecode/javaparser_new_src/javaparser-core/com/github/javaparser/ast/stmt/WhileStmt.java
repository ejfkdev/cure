package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithBody;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class WhileStmt extends Statement implements NodeWithBody<WhileStmt> {
    private Expression condition;
    private Statement body;
    public WhileStmt() {}
    public WhileStmt(final Expression condition, final Statement body) {
        setCondition(condition);
        setBody(body);
    }
    public WhileStmt(Range range, final Expression condition, final Statement body) {
        super(range);
        setCondition(condition);
        setBody(body);
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
    public WhileStmt setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
        return this;
    }
    public WhileStmt setCondition(final Expression condition) {
        this.condition = condition;
        setAsParentNodeOf(this.condition);
        return this;
    }
}
