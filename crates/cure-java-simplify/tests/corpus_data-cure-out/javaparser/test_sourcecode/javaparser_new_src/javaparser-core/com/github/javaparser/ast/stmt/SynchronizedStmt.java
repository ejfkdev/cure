package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithBlockStmt;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class SynchronizedStmt extends Statement implements NodeWithBlockStmt<SynchronizedStmt> {
    private Expression expr;
    private BlockStmt block;
    public SynchronizedStmt() {}
    public SynchronizedStmt(final Expression expr, final BlockStmt block) {
        setExpr(expr);
        setBlock(block);
    }
    public SynchronizedStmt(Range range, final Expression expr, final BlockStmt block) {
        super(range);
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
    @Deprecated
    public BlockStmt getBlock() {
        return block;
    }
    public Expression getExpr() {
        return expr;
    }
    @Deprecated
    public SynchronizedStmt setBlock(final BlockStmt block) {
        this.block = block;
        setAsParentNodeOf(this.block);
        return this;
    }
    public SynchronizedStmt setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
        return this;
    }
    @Override
    public BlockStmt getBody() {
        return block;
    }
    @Override
    public SynchronizedStmt setBody(BlockStmt block) {
        this.block = block;
        setAsParentNodeOf(this.block);
        return this;
    }
}
