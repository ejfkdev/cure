package com.github.javaparser.ast.body;

import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.NamedNode;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class AnnotationMemberDeclaration extends BodyDeclaration implements DocumentableNode, NamedNode {
    private int modifiers;
    private Type type;
    private String name;
    private Expression defaultValue;
    public AnnotationMemberDeclaration() {}
    public AnnotationMemberDeclaration(int modifiers, Type type, String name, Expression defaultValue) {
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setDefaultValue(defaultValue);
    }
    public AnnotationMemberDeclaration(int modifiers, List<AnnotationExpr> annotations, Type type, String name, Expression defaultValue) {
        super(annotations);
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setDefaultValue(defaultValue);
    }
    public AnnotationMemberDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, int modifiers, List<AnnotationExpr> annotations, Type type, String name, Expression defaultValue) {
        super(beginLine, beginColumn, endLine, endColumn, annotations);
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setDefaultValue(defaultValue);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Expression getDefaultValue() {
        return defaultValue;
    }
    public int getModifiers() {
        return modifiers;
    }
    public String getName() {
        return name;
    }
    public Type getType() {
        return type;
    }
    public void setDefaultValue(Expression defaultValue) {
        this.defaultValue = defaultValue;
        setAsParentNodeOf(defaultValue);
    }
    public void setModifiers(int modifiers) {
        this.modifiers = modifiers;
    }
    public void setName(String name) {
        this.name = name;
    }
    public void setType(Type type) {
        this.type = type;
        setAsParentNodeOf(type);
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
