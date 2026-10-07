package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class TypeParameter extends ReferenceType<TypeParameter> implements NodeWithName<TypeParameter> {
    private String name;
    private List<AnnotationExpr> annotations;
    private List<ClassOrInterfaceType> typeBound;
    public TypeParameter() {}
    public TypeParameter(final String name, final List<ClassOrInterfaceType> typeBound) {
        setName(name);
        setTypeBound(typeBound);
    }
    public TypeParameter(Range range, final String name, final List<ClassOrInterfaceType> typeBound) {
        super(range);
        setName(name);
        setTypeBound(typeBound);
    }
    public TypeParameter(Range range, String name, List<ClassOrInterfaceType> typeBound, List<AnnotationExpr> annotations) {
        this(range, name, typeBound);
        setTypeBound(typeBound);
        setAnnotations(annotations);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    @Override
	public String getName() {
        return name;
    }
    public List<ClassOrInterfaceType> getTypeBound() {
        typeBound = ensureNotNull(typeBound);
        return typeBound;
    }
    @Override
    public TypeParameter setName(final String name) {
        this.name = name;
        return this;
    }
    public TypeParameter setTypeBound(final List<ClassOrInterfaceType> typeBound) {
        this.typeBound = typeBound;
        setAsParentNodeOf(typeBound);
        return this;
    }
    public List<AnnotationExpr> getAnnotations() {
        annotations = ensureNotNull(annotations);
        return annotations;
    }
    public TypeParameter setAnnotations(List<AnnotationExpr> annotations) {
        this.annotations = annotations;
        setAsParentNodeOf(this.annotations);
        return this;
    }
}
