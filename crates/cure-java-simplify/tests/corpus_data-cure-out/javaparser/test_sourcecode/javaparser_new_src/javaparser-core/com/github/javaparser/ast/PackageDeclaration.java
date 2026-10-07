package com.github.javaparser.ast;

import com.github.javaparser.Range;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.utils.Utils;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class PackageDeclaration extends Node implements NodeWithAnnotations<PackageDeclaration> {
    private List<AnnotationExpr> annotations;
    private NameExpr name;
    public PackageDeclaration() {}
    public PackageDeclaration(NameExpr name) {
        setName(name);
    }
    public PackageDeclaration(List<AnnotationExpr> annotations, NameExpr name) {
        setAnnotations(annotations);
        setName(name);
    }
    public PackageDeclaration(Range range, List<AnnotationExpr> annotations, NameExpr name) {
        super(range);
        setAnnotations(annotations);
        setName(name);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public List<AnnotationExpr> getAnnotations() {
        annotations = Utils.ensureNotNull(annotations);
        return annotations;
    }
    public NameExpr getName() {
        return name;
    }
    public String getPackageName() {
        return name.toString();
    }
    public PackageDeclaration setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
        setAsParentNodeOf(this.annotations);
        return this;
    }
    public PackageDeclaration setName(NameExpr name) {
        this.name = name;
        setAsParentNodeOf(this.name);
        return this;
    }
}
