package com.github.javaparser.ast.body;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class VariableDeclarator extends Node {
    private VariableDeclaratorId id;
    private Expression init;
    public VariableDeclarator() {}
    public VariableDeclarator(VariableDeclaratorId id) {
        setId(id);
    }
    public VariableDeclarator(VariableDeclaratorId id, Expression init) {
        setId(id);
        setInit(init);
    }
    public VariableDeclarator(int beginLine, int beginColumn, int endLine, int endColumn, VariableDeclaratorId id, Expression init) {
        super(beginLine, beginColumn, endLine, endColumn);
        setId(id);
        setInit(init);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public VariableDeclaratorId getId() {
        return id;
    }
    public Expression getInit() {
        return init;
    }
    public void setId(VariableDeclaratorId id) {
        this.id = id;
        setAsParentNodeOf(this.id);
    }
    public void setInit(Expression init) {
        this.init = init;
        setAsParentNodeOf(this.init);
    }
}
