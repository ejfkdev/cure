package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class EnumDeclaration extends TypeDeclaration implements DocumentableNode {
    private List<ClassOrInterfaceType> implementsList;
    private List<EnumConstantDeclaration> entries;
    public EnumDeclaration() {}
    public EnumDeclaration(int modifiers, String name) {
        super(modifiers, name);
    }
    public EnumDeclaration(int modifiers, List<AnnotationExpr> annotations, String name, List<ClassOrInterfaceType> implementsList, List<EnumConstantDeclaration> entries, List<BodyDeclaration> members) {
        super(annotations, modifiers, name, members);
        setImplements(implementsList);
        setEntries(entries);
    }
    public EnumDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, int modifiers, List<AnnotationExpr> annotations, String name, List<ClassOrInterfaceType> implementsList, List<EnumConstantDeclaration> entries, List<BodyDeclaration> members) {
        super(beginLine, beginColumn, endLine, endColumn, annotations, modifiers, name, members);
        setImplements(implementsList);
        setEntries(entries);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public List<EnumConstantDeclaration> getEntries() {
        return entries;
    }
    public List<ClassOrInterfaceType> getImplements() {
        return implementsList;
    }
    public void setEntries(List<EnumConstantDeclaration> entries) {
        this.entries = entries;
        setAsParentNodeOf(this.entries);
    }
    public void setImplements(List<ClassOrInterfaceType> implementsList) {
        this.implementsList = implementsList;
        setAsParentNodeOf(this.implementsList);
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
