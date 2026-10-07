package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class SwitchEntryStmt extends Statement {
    private Expression label;
    private List<Statement> stmts;
    public SwitchEntryStmt() {}
    public SwitchEntryStmt(final Expression label, final List<Statement> stmts) {
        setLabel(label);
        setStmts(stmts);
    }
    public SwitchEntryStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression label, final List<Statement> stmts) {
        super(beginLine, beginColumn, endLine, endColumn);
        setLabel(label);
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
    public Expression getLabel() {
        return label;
    }
    public List<Statement> getStmts() {
        return stmts;
    }
    public void setLabel(final Expression label) {
        this.label = label;
        setAsParentNodeOf(this.label);
    }
    public void setStmts(final List<Statement> stmts) {
        this.stmts = stmts;
        setAsParentNodeOf(this.stmts);
    }
}
