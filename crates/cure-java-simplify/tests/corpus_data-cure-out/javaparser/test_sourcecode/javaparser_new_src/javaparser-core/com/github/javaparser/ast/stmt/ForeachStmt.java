package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithBody;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ForeachStmt extends Statement implements NodeWithBody<ForeachStmt> {
    private VariableDeclarationExpr var;
    private Expression iterable;
    private Statement body;
    public ForeachStmt() {}
    public ForeachStmt(final VariableDeclarationExpr var, final Expression iterable, final Statement body) {
        setVariable(var);
        setIterable(iterable);
        setBody(body);
    }
    public ForeachStmt(Range range, final VariableDeclarationExpr var, final Expression iterable, final Statement body) {
        super(range);
        setVariable(var);
        setIterable(iterable);
        setBody(body);
    }
    public ForeachStmt(VariableDeclarationExpr var, String iterable, BlockStmt body) {
        setVariable(var);
        setIterable(new NameExpr(iterable));
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
    @Override
    public Statement getBody() {
        return body;
    }
    public Expression getIterable() {
        return iterable;
    }
    public VariableDeclarationExpr getVariable() {
        return var;
    }
    @Override
    public ForeachStmt setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
        return this;
    }
    public ForeachStmt setIterable(final Expression iterable) {
        this.iterable = iterable;
        setAsParentNodeOf(this.iterable);
        return this;
    }
    public ForeachStmt setVariable(final VariableDeclarationExpr var) {
        this.var = var;
        setAsParentNodeOf(this.var);
        return this;
    }
}
