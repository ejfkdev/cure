package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class SwitchStmt extends Statement {
    private Expression selector;
    private List<SwitchEntryStmt> entries;
    public SwitchStmt() {}
    public SwitchStmt(final Expression selector, final List<SwitchEntryStmt> entries) {
        setSelector(selector);
        setEntries(entries);
    }
    public SwitchStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression selector, final List<SwitchEntryStmt> entries) {
        super(beginLine, beginColumn, endLine, endColumn);
        setSelector(selector);
        setEntries(entries);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<SwitchEntryStmt> getEntries() {
        return entries;
    }
    public Expression getSelector() {
        return selector;
    }
    public void setEntries(final List<SwitchEntryStmt> entries) {
        this.entries = entries;
        setAsParentNodeOf(this.entries);
    }
    public void setSelector(final Expression selector) {
        this.selector = selector;
        setAsParentNodeOf(this.selector);
    }
}
