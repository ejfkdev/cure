package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class EmptyTypeDeclaration extends TypeDeclaration implements DocumentableNode {
    public EmptyTypeDeclaration() {
        super(null, 0, null, null);
    }
    public EmptyTypeDeclaration(int beginLine, int beginColumn, int endLine, int endColumn) {
        super(beginLine, beginColumn, endLine, endColumn, null, 0, null, null);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public void setJavaDoc(JavadocComment javadocComment) {}
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    @Override
    public JavadocComment getJavaDoc() {
        return null;
    }
}
