package com.github.javaparser.ast.body;

import com.github.javaparser.ast.NamedNode;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class VariableDeclaratorId extends Node implements NamedNode {
    private String name;
    private int arrayCount;
    public VariableDeclaratorId() {}
    public VariableDeclaratorId(String name) {
        setName(name);
    }
    public VariableDeclaratorId(int beginLine, int beginColumn, int endLine, int endColumn, String name, int arrayCount) {
        super(beginLine, beginColumn, endLine, endColumn);
        setName(name);
        setArrayCount(arrayCount);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public int getArrayCount() {
        return arrayCount;
    }
    public String getName() {
        return name;
    }
    public void setArrayCount(int arrayCount) {
        this.arrayCount = arrayCount;
    }
    public void setName(String name) {
        this.name = name;
    }
}
