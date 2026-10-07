package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import java.util.List;

public final class TryStmt extends Statement {
    private List<VariableDeclarationExpr> resources;
    private BlockStmt tryBlock;
    private List<CatchClause> catchs;
    private BlockStmt finallyBlock;
    public TryStmt() {}
    public TryStmt(final BlockStmt tryBlock, final List<CatchClause> catchs, final BlockStmt finallyBlock) {
        setTryBlock(tryBlock);
        setCatchs(catchs);
        setFinallyBlock(finallyBlock);
    }
    public TryStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, List<VariableDeclarationExpr> resources, final BlockStmt tryBlock, final List<CatchClause> catchs, final BlockStmt finallyBlock) {
        super(beginLine, beginColumn, endLine, endColumn);
        setResources(resources);
        setTryBlock(tryBlock);
        setCatchs(catchs);
        setFinallyBlock(finallyBlock);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<CatchClause> getCatchs() {
        return catchs;
    }
    public BlockStmt getFinallyBlock() {
        return finallyBlock;
    }
    public BlockStmt getTryBlock() {
        return tryBlock;
    }
    public List<VariableDeclarationExpr> getResources() {
        return resources;
    }
    public void setCatchs(final List<CatchClause> catchs) {
        this.catchs = catchs;
        setAsParentNodeOf(this.catchs);
    }
    public void setFinallyBlock(final BlockStmt finallyBlock) {
        this.finallyBlock = finallyBlock;
        setAsParentNodeOf(this.finallyBlock);
    }
    public void setTryBlock(final BlockStmt tryBlock) {
        this.tryBlock = tryBlock;
        setAsParentNodeOf(this.tryBlock);
    }
    public void setResources(List<VariableDeclarationExpr> resources) {
        this.resources = resources;
        setAsParentNodeOf(this.resources);
    }
}
