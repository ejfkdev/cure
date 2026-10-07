package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithTypeArguments;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public class MethodReferenceExpr extends Expression implements NodeWithTypeArguments<MethodReferenceExpr> {
    private Expression scope;
    private List<Type<?>> typeArguments;
    private String identifier;
    public MethodReferenceExpr() {}
    public MethodReferenceExpr(Range range, Expression scope, List<Type<?>> typeArguments, String identifier) {
        super(range);
        setIdentifier(identifier);
        setScope(scope);
        setTypeArguments(typeArguments);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public Expression getScope() {
        return scope;
    }
    public MethodReferenceExpr setScope(Expression scope) {
        this.scope = scope;
        setAsParentNodeOf(this.scope);
        return this;
    }
    @Override
    public List<Type<?>> getTypeArguments() {
        return typeArguments;
    }
    @Override
    public MethodReferenceExpr setTypeArguments(final List<Type<?>> types) {
        this.typeArguments = types;
        setAsParentNodeOf(this.typeArguments);
        return this;
    }
    public String getIdentifier() {
        return identifier;
    }
    public MethodReferenceExpr setIdentifier(String identifier) {
        this.identifier = identifier;
        return this;
    }
}
