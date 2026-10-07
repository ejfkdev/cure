package com.github.javaparser.ast.stmt;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.MultiTypeParameter;
import com.github.javaparser.ast.body.VariableDeclaratorId;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class CatchClause extends Node {
    private MultiTypeParameter except;
    private BlockStmt catchBlock;
    public CatchClause() {}
    public CatchClause(final MultiTypeParameter except, final BlockStmt catchBlock) {
        setExcept(except);
        setCatchBlock(catchBlock);
    }
    public CatchClause(int exceptModifier, List<AnnotationExpr> exceptAnnotations, List<Type> exceptTypes, VariableDeclaratorId exceptId, BlockStmt catchBlock) {
        this(new MultiTypeParameter(exceptModifier, exceptAnnotations, exceptTypes, exceptId), catchBlock);
    }
    public CatchClause(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final int exceptModifier, final List<AnnotationExpr> exceptAnnotations, final List<Type> exceptTypes, final VariableDeclaratorId exceptId, final BlockStmt catchBlock) {
        super(beginLine, beginColumn, endLine, endColumn);
        setExcept(new MultiTypeParameter(beginLine, beginColumn, endLine, endColumn, exceptModifier, exceptAnnotations, exceptTypes, exceptId));
        setCatchBlock(catchBlock);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public BlockStmt getCatchBlock() {
        return catchBlock;
    }
    public MultiTypeParameter getExcept() {
        return except;
    }
    public void setCatchBlock(final BlockStmt catchBlock) {
        this.catchBlock = catchBlock;
        setAsParentNodeOf(this.catchBlock);
    }
    public void setExcept(final MultiTypeParameter except) {
        this.except = except;
        setAsParentNodeOf(this.except);
    }
}
