package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class BlockStmt extends Statement {
    private List<Statement> stmts;
    public BlockStmt() {}
    public BlockStmt(final List<Statement> stmts) {
        setStmts(stmts);
    }
    public BlockStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final List<Statement> stmts) {
        super(beginLine, beginColumn, endLine, endColumn);
        setStmts(stmts);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<Statement> getStmts() {
        return stmts;
    }
    public void setStmts(final List<Statement> stmts) {
        this.stmts = stmts;
        setAsParentNodeOf(this.stmts);
    }
}
