package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class InitializerDeclaration extends BodyDeclaration implements DocumentableNode {
    private boolean isStatic;
    private BlockStmt block;
    public InitializerDeclaration() {}
    public InitializerDeclaration(boolean isStatic, BlockStmt block) {
        super(null);
        setStatic(isStatic);
        setBlock(block);
    }
    public InitializerDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, boolean isStatic, BlockStmt block) {
        super(beginLine, beginColumn, endLine, endColumn, null);
        setStatic(isStatic);
        setBlock(block);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public BlockStmt getBlock() {
        return block;
    }
    public boolean isStatic() {
        return isStatic;
    }
    public void setBlock(BlockStmt block) {
        this.block = block;
        setAsParentNodeOf(this.block);
    }
    public void setStatic(boolean isStatic) {
        this.isStatic = isStatic;
    }
    @Override
    public void setJavaDoc(JavadocComment javadocComment) {
        this.javadocComment = javadocComment;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return javadocComment;
    }
    private JavadocComment javadocComment;
}
