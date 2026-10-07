package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class ReturnStmt extends Statement {
    private Expression expr;
    public ReturnStmt() {}
    public ReturnStmt(final Expression expr) {
        setExpr(expr);
    }
    public ReturnStmt(Range range, final Expression expr) {
        super(range);
        setExpr(expr);
    }
    public ReturnStmt(String expr) {
        setExpr(new NameExpr(expr));
    }
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Expression getExpr() {
        return expr;
    }
    public ReturnStmt setExpr(final Expression expr) {
        this.expr = expr;
        setAsParentNodeOf(this.expr);
        return this;
    }
}
