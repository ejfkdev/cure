package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class SynchronizedStmt extends Statement {
    private Expression expr;
    private BlockStmt block;
    public SynchronizedStmt() {}
    public SynchronizedStmt(final Expression expr, final BlockStmt block) {
        setExpr(expr);
        setBlock(block);
    }
    public SynchronizedStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression expr, final BlockStmt block) {
        super(beginLine, beginColumn, endLine, endColumn);
        setExpr(expr);
        setBlock(block);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public BlockStmt getBlock() {
        return block;
    }
    public Expression getExpr() {
        return expr;
    }
    public void setBlock(final BlockStmt block) {
        this.block = block;
        setAsParentNodeOf(this.block);
    }
    public void setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
}
