package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithStatements;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class SwitchEntryStmt extends Statement implements NodeWithStatements<SwitchEntryStmt> {
    private Expression label;
    private List<Statement> stmts;
    public SwitchEntryStmt() {}
    public SwitchEntryStmt(final Expression label, final List<Statement> stmts) {
        setLabel(label);
        setStmts(stmts);
    }
    public SwitchEntryStmt(Range range, final Expression label, final List<Statement> stmts) {
        super(range);
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
    @Override
    public List<Statement> getStmts() {
        stmts = ensureNotNull(stmts);
        return stmts;
    }
    public SwitchEntryStmt setLabel(final Expression label) {
        this.label = label;
        setAsParentNodeOf(this.label);
        return this;
    }
    @Override
    public SwitchEntryStmt setStmts(final List<Statement> stmts) {
        this.stmts = stmts;
        setAsParentNodeOf(this.stmts);
        return this;
    }
}
