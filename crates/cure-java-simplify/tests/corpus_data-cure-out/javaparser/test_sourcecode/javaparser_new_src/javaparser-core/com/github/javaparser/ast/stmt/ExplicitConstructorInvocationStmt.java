package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithTypeArguments;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class ExplicitConstructorInvocationStmt extends Statement implements NodeWithTypeArguments<ExplicitConstructorInvocationStmt> {
    private List<Type<?>> typeArguments;
    private boolean isThis;
    private Expression expr;
    private List<Expression> args;
    public ExplicitConstructorInvocationStmt() {}
    public ExplicitConstructorInvocationStmt(final boolean isThis, final Expression expr, final List<Expression> args) {
        setThis(isThis);
        setExpr(expr);
        setArgs(args);
    }
    public ExplicitConstructorInvocationStmt(Range range, final List<Type<?>> typeArguments, final boolean isThis, final Expression expr, final List<Expression> args) {
        super(range);
        setTypeArguments(typeArguments);
        setThis(isThis);
        setExpr(expr);
        setArgs(args);
    }
    @Override
	public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<Expression> getArgs() {
        args = ensureNotNull(args);
        return args;
    }
    public Expression getExpr() {
        return expr;
    }
    public boolean isThis() {
        return isThis;
    }
    public ExplicitConstructorInvocationStmt setArgs(final List<Expression> args) {
        this.args = args;
        setAsParentNodeOf(this.args);
        return this;
    }
    public ExplicitConstructorInvocationStmt setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
        return this;
    }
    public ExplicitConstructorInvocationStmt setThis(final boolean isThis) {
        this.isThis = isThis;
        return this;
    }
    @Override
    public List<Type<?>> getTypeArguments() {
        return typeArguments;
    }
    @Override
    public ExplicitConstructorInvocationStmt setTypeArguments(final List<Type<?>> types) {
        this.typeArguments = types;
        setAsParentNodeOf(this.typeArguments);
        return this;
    }
}
