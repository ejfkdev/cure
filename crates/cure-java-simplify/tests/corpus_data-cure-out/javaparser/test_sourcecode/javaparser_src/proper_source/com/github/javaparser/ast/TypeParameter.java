package com.github.javaparser.ast;

import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class TypeParameter extends Node implements NamedNode {
    private String name;
    private List<AnnotationExpr> annotations;
    private List<ClassOrInterfaceType> typeBound;
    public TypeParameter() {}
    public TypeParameter(final String name, final List<ClassOrInterfaceType> typeBound) {
        setName(name);
        setTypeBound(typeBound);
    }
    public TypeParameter(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final String name, final List<ClassOrInterfaceType> typeBound) {
        super(beginLine, beginColumn, endLine, endColumn);
        setName(name);
        setTypeBound(typeBound);
    }
    public TypeParameter(int beginLine, int beginColumn, int endLine, int endColumn, String name, List<ClassOrInterfaceType> typeBound, List<AnnotationExpr> annotations) {
        this(beginLine, beginColumn, endLine, endColumn, name, typeBound);
        setName(name);
        setTypeBound(typeBound);
        this.annotations = annotations;
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public String getName() {
        return name;
    }
    public List<ClassOrInterfaceType> getTypeBound() {
        return typeBound;
    }
    public void setName(final String name) {
        this.name = name;
    }
    public void setTypeBound(final List<ClassOrInterfaceType> typeBound) {
        this.typeBound = typeBound;
        setAsParentNodeOf(typeBound);
    }
    public List<AnnotationExpr> getAnnotations() {
        return annotations;
    }
    public void setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
    }
}
