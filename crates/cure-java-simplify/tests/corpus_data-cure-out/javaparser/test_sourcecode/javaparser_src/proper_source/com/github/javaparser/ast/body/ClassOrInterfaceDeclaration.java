package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.TypeParameter;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ClassOrInterfaceDeclaration extends TypeDeclaration implements DocumentableNode {
    private boolean interface_;
    private List<TypeParameter> typeParameters;
    private List<ClassOrInterfaceType> extendsList;
    private List<ClassOrInterfaceType> implementsList;
    public ClassOrInterfaceDeclaration() {}
    public ClassOrInterfaceDeclaration(final int modifiers, final boolean isInterface, final String name) {
        super(modifiers, name);
        setInterface(isInterface);
    }
    public ClassOrInterfaceDeclaration(final int modifiers, final List<AnnotationExpr> annotations, final boolean isInterface, final String name, final List<TypeParameter> typeParameters, final List<ClassOrInterfaceType> extendsList, final List<ClassOrInterfaceType> implementsList, final List<BodyDeclaration> members) {
        super(annotations, modifiers, name, members);
        setInterface(isInterface);
        setTypeParameters(typeParameters);
        setExtends(extendsList);
        setImplements(implementsList);
    }
    public ClassOrInterfaceDeclaration(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final int modifiers, final List<AnnotationExpr> annotations, final boolean isInterface, final String name, final List<TypeParameter> typeParameters, final List<ClassOrInterfaceType> extendsList, final List<ClassOrInterfaceType> implementsList, final List<BodyDeclaration> members) {
        super(beginLine, beginColumn, endLine, endColumn, annotations, modifiers, name, members);
        setInterface(isInterface);
        setTypeParameters(typeParameters);
        setExtends(extendsList);
        setImplements(implementsList);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<ClassOrInterfaceType> getExtends() {
        return extendsList;
    }
    public List<ClassOrInterfaceType> getImplements() {
        return implementsList;
    }
    public List<TypeParameter> getTypeParameters() {
        return typeParameters;
    }
    public boolean isInterface() {
        return interface_;
    }
    public void setExtends(final List<ClassOrInterfaceType> extendsList) {
        this.extendsList = extendsList;
        setAsParentNodeOf(this.extendsList);
    }
    public void setImplements(final List<ClassOrInterfaceType> implementsList) {
        this.implementsList = implementsList;
        setAsParentNodeOf(this.implementsList);
    }
    public void setInterface(final boolean interface_) {
        this.interface_ = interface_;
    }
    public void setTypeParameters(final List<TypeParameter> typeParameters) {
        this.typeParameters = typeParameters;
        setAsParentNodeOf(this.typeParameters);
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
