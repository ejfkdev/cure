package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.nodeTypes.NodeWithTypeArguments;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class ClassOrInterfaceType extends ReferenceType<ClassOrInterfaceType> implements NodeWithName<ClassOrInterfaceType>, NodeWithAnnotations<ClassOrInterfaceType>, NodeWithTypeArguments<ClassOrInterfaceType> {
    private ClassOrInterfaceType scope;
    private String name;
    private List<Type<?>> typeArguments;
    public ClassOrInterfaceType() {}
    public ClassOrInterfaceType(final String name) {
        setName(name);
    }
    public ClassOrInterfaceType(final ClassOrInterfaceType scope, final String name) {
        setScope(scope);
        setName(name);
    }
    public ClassOrInterfaceType(final Range range, final ClassOrInterfaceType scope, final String name, final List<Type<?>> typeArguments) {
        super(range);
        setScope(scope);
        setName(name);
        setTypeArguments(typeArguments);
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
    public ClassOrInterfaceType getScope() {
        return scope;
    }
    public boolean isBoxedType() {
        return PrimitiveType.unboxMap.containsKey(name);
    }
    public PrimitiveType toUnboxedType() throws UnsupportedOperationException {
        if (!isBoxedType()) {
            throw new UnsupportedOperationException(name + " isn't a boxed type.");
        }
        return new PrimitiveType(PrimitiveType.unboxMap.get(name));
    }
    @Override
    public ClassOrInterfaceType setName(final String name) {
        this.name = name;
        return this;
    }
    public ClassOrInterfaceType setScope(final ClassOrInterfaceType scope) {
        this.scope = scope;
        setAsParentNodeOf(this.scope);
        return this;
    }
    @Override
    public List<Type<?>> getTypeArguments() {
        return typeArguments;
    }
    @Override
    public ClassOrInterfaceType setTypeArguments(final List<Type<?>> types) {
        this.typeArguments = types;
        setAsParentNodeOf(this.typeArguments);
        return this;
    }
}
