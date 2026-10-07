package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class AnnotationDeclaration extends TypeDeclaration implements DocumentableNode {
    public AnnotationDeclaration() {}
    public AnnotationDeclaration(int modifiers, String name) {
        super(modifiers, name);
    }
    public AnnotationDeclaration(int modifiers, List<AnnotationExpr> annotations, String name, List<BodyDeclaration> members) {
        super(annotations, modifiers, name, members);
    }
    public AnnotationDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, int modifiers, List<AnnotationExpr> annotations, String name, List<BodyDeclaration> members) {
        super(beginLine, beginColumn, endLine, endColumn, annotations, modifiers, name, members);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
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
