package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class CharLiteralExpr extends StringLiteralExpr {
    public CharLiteralExpr() {}
    public CharLiteralExpr(String value) {
        super(value);
    }
    public CharLiteralExpr(int beginLine, int beginColumn, int endLine, int endColumn, String value) {
        super(beginLine, beginColumn, endLine, endColumn, value);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
}
