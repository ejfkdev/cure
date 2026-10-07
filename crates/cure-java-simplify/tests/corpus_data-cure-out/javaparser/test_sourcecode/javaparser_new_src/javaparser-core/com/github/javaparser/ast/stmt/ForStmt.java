package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithBody;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class ForStmt extends Statement implements NodeWithBody<ForStmt> {
    private List<Expression> init;
    private Expression compare;
    private List<Expression> update;
    private Statement body;
    public ForStmt() {}
    public ForStmt(final List<Expression> init, final Expression compare, final List<Expression> update, final Statement body) {
        setCompare(compare);
        setInit(init);
        setUpdate(update);
        setBody(body);
    }
    public ForStmt(Range range, final List<Expression> init, final Expression compare, final List<Expression> update, final Statement body) {
        super(range);
        setCompare(compare);
        setInit(init);
        setUpdate(update);
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
    public Expression getCompare() {
        return compare;
    }
    public List<Expression> getInit() {
        init = ensureNotNull(init);
        return init;
    }
    public List<Expression> getUpdate() {
        update = ensureNotNull(update);
        return update;
    }
    @Override
    public ForStmt setBody(final Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
        return this;
    }
    public ForStmt setCompare(final Expression compare) {
        this.compare = compare;
        setAsParentNodeOf(this.compare);
        return this;
    }
    public ForStmt setInit(final List<Expression> init) {
        this.init = init;
        setAsParentNodeOf(this.init);
        return this;
    }
    public ForStmt setUpdate(final List<Expression> update) {
        this.update = update;
        setAsParentNodeOf(this.update);
        return this;
    }
}
