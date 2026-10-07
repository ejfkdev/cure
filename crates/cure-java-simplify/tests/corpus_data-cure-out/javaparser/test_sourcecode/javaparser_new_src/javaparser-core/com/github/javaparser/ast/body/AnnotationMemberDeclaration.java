package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.nodeTypes.NodeWithJavaDoc;
import com.github.javaparser.ast.nodeTypes.NodeWithModifiers;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.nodeTypes.NodeWithType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.EnumSet;
import java.util.List;

public final class AnnotationMemberDeclaration extends BodyDeclaration<AnnotationMemberDeclaration> implements NodeWithJavaDoc<AnnotationMemberDeclaration>, NodeWithName<AnnotationMemberDeclaration>, NodeWithType<AnnotationMemberDeclaration>, NodeWithModifiers<AnnotationMemberDeclaration> {
    private EnumSet<Modifier> modifiers = EnumSet.noneOf(Modifier.class);
    private Type type;
    private String name;
    private Expression defaultValue;
    public AnnotationMemberDeclaration() {}
    public AnnotationMemberDeclaration(EnumSet<Modifier> modifiers, Type type, String name, Expression defaultValue) {
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setDefaultValue(defaultValue);
    }
    public AnnotationMemberDeclaration(EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, Type type, String name, Expression defaultValue) {
        super(annotations);
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setDefaultValue(defaultValue);
    }
    public AnnotationMemberDeclaration(Range range, EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, Type type, String name, Expression defaultValue) {
        super(range, annotations);
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
    @Override
    public EnumSet<Modifier> getModifiers() {
        return modifiers;
    }
    @Override
    public String getName() {
        return name;
    }
    @Override
    public Type getType() {
        return type;
    }
    public AnnotationMemberDeclaration setDefaultValue(Expression defaultValue) {
        this.defaultValue = defaultValue;
        setAsParentNodeOf(defaultValue);
        return this;
    }
    @Override
    public AnnotationMemberDeclaration setModifiers(EnumSet<Modifier> modifiers) {
        this.modifiers = modifiers;
        return this;
    }
    @Override
    public AnnotationMemberDeclaration setName(String name) {
        this.name = name;
        return this;
    }
    @Override
    public AnnotationMemberDeclaration setType(Type type) {
        this.type = type;
        setAsParentNodeOf(type);
        return this;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
}
