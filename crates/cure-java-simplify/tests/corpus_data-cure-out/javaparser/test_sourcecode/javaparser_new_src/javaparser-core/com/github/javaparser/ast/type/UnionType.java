package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public class UnionType extends Type<UnionType> implements NodeWithAnnotations<UnionType> {
    private List<ReferenceType> elements;
    public UnionType(Range range, List<ReferenceType> elements) {
        super(range);
        setElements(elements);
    }
    public UnionType(List<ReferenceType> elements) {
        super();
        setElements(elements);
    }
    public List<ReferenceType> getElements() {
        return elements;
    }
    public UnionType setElements(List<ReferenceType> elements) {
        if (this.elements != null) {
            for (ReferenceType element : elements) {
                element.setParentNode(null);
            }
        }
        this.elements = elements;
        setAsParentNodeOf(this.elements);
        return this;
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
}
