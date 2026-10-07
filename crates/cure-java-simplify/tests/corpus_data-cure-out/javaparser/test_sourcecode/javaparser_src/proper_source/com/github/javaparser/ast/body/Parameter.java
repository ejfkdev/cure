package com.github.javaparser.ast.body;

import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class Parameter extends BaseParameter {
    private Type type;
    private boolean isVarArgs;
    public Parameter() {}
    public Parameter(Type type, VariableDeclaratorId id) {
        super(id);
        setType(type);
    }
    public Parameter(int modifiers, Type type, VariableDeclaratorId id) {
        super(modifiers, id);
        setType(type);
    }
    public Parameter(int beginLine, int beginColumn, int endLine, int endColumn, int modifiers, List<AnnotationExpr> annotations, Type type, boolean isVarArgs, VariableDeclaratorId id) {
        super(beginLine, beginColumn, endLine, endColumn, modifiers, annotations, id);
        setType(type);
        setVarArgs(isVarArgs);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Type getType() {
        return type;
    }
    public boolean isVarArgs() {
        return isVarArgs;
    }
    public void setType(Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
    public void setVarArgs(boolean isVarArgs) {
        this.isVarArgs = isVarArgs;
    }
}
