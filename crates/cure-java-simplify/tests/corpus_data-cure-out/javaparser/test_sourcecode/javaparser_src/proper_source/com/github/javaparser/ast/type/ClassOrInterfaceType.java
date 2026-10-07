package com.github.javaparser.ast.type;

import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ClassOrInterfaceType extends Type {
    private ClassOrInterfaceType scope;
    private String name;
    private List<Type> typeArgs;
    public ClassOrInterfaceType() {}
    public ClassOrInterfaceType(final String name) {
        setName(name);
    }
    public ClassOrInterfaceType(final ClassOrInterfaceType scope, final String name) {
        setScope(scope);
        setName(name);
    }
    public ClassOrInterfaceType(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final ClassOrInterfaceType scope, final String name, final List<Type> typeArgs) {
        super(beginLine, beginColumn, endLine, endColumn);
        setScope(scope);
        setName(name);
        setTypeArgs(typeArgs);
    }
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public String getName() {
        return name;
    }
    public ClassOrInterfaceType getScope() {
        return scope;
    }
    public List<Type> getTypeArgs() {
        return typeArgs;
    }
    public boolean isBoxedType() {
        return PrimitiveType.unboxMap.containsKey(name);
    }
    public PrimitiveType toUnboxedType() throws UnsupportedOperationException {
        if (!isBoxedType()) 
            throw new UnsupportedOperationException(name + " isn't a boxed type.");
        return new PrimitiveType(PrimitiveType.unboxMap.get(name));
    }
    public void setName(final String name) {
        this.name = name;
    }
    public void setScope(final ClassOrInterfaceType scope) {
        this.scope = scope;
        setAsParentNodeOf(this.scope);
    }
    public void setTypeArgs(final List<Type> typeArgs) {
        this.typeArgs = typeArgs;
        setAsParentNodeOf(this.typeArgs);
    }
}
