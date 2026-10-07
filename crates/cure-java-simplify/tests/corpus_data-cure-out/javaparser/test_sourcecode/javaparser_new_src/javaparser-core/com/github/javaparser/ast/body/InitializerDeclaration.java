package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.nodeTypes.NodeWithJavaDoc;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class InitializerDeclaration extends BodyDeclaration<InitializerDeclaration> implements NodeWithJavaDoc<InitializerDeclaration> {
    private boolean isStatic;
    private BlockStmt block;
    public InitializerDeclaration() {}
    public InitializerDeclaration(boolean isStatic, BlockStmt block) {
        super(null);
        setStatic(isStatic);
        setBlock(block);
    }
    public InitializerDeclaration(Range range, boolean isStatic, BlockStmt block) {
        super(range, null);
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
    public InitializerDeclaration setBlock(BlockStmt block) {
        this.block = block;
        setAsParentNodeOf(this.block);
        return this;
    }
    public InitializerDeclaration setStatic(boolean isStatic) {
        this.isStatic = isStatic;
        return this;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
}
