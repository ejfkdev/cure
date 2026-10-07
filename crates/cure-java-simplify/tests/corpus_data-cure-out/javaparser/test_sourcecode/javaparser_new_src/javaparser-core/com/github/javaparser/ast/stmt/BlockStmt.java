package com.github.javaparser.ast.stmt;

import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithStatements;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class BlockStmt extends Statement implements NodeWithStatements<BlockStmt> {
    private List<Statement> stmts;
    public BlockStmt() {}
    public BlockStmt(final List<Statement> stmts) {
        setStmts(stmts);
    }
    public BlockStmt(final Range range, final List<Statement> stmts) {
        super(range);
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
    @Override
    public List<Statement> getStmts() {
        stmts = ensureNotNull(stmts);
        return stmts;
    }
    @Override
    public BlockStmt setStmts(final List<Statement> stmts) {
        this.stmts = stmts;
        setAsParentNodeOf(this.stmts);
        return this;
    }
}
