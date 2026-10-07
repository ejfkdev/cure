package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ExplicitConstructorInvocationStmt extends Statement {
    private List<Type> typeArgs;
    private boolean isThis;
    private Expression expr;
    private List<Expression> args;
    public ExplicitConstructorInvocationStmt() {}
    public ExplicitConstructorInvocationStmt(final boolean isThis, final Expression expr, final List<Expression> args) {
        setThis(isThis);
        setExpr(expr);
        setArgs(args);
    }
    public ExplicitConstructorInvocationStmt(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final List<Type> typeArgs, final boolean isThis, final Expression expr, final List<Expression> args) {
        super(beginLine, beginColumn, endLine, endColumn);
        setTypeArgs(typeArgs);
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
        return args;
    }
    public Expression getExpr() {
        return expr;
    }
    public List<Type> getTypeArgs() {
        return typeArgs;
    }
    public boolean isThis() {
        return isThis;
    }
    public void setArgs(final List<Expression> args) {
        this.args = args;
        setAsParentNodeOf(this.args);
    }
    public void setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
    }
    public void setThis(final boolean isThis) {
        this.isThis = isThis;
    }
    public void setTypeArgs(final List<Type> typeArgs) {
        this.typeArgs = typeArgs;
        setAsParentNodeOf(this.typeArgs);
    }
}
