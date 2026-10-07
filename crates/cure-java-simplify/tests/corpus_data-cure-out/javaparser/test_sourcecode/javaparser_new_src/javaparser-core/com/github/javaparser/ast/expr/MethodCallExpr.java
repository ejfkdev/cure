package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithTypeArguments;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class MethodCallExpr extends Expression implements NodeWithTypeArguments<MethodCallExpr> {
    private Expression scope;
    private List<Type<?>> typeArguments;
    private NameExpr name;
    private List<Expression> args;
    public MethodCallExpr() {}
    public MethodCallExpr(final Expression scope, final String name) {
        setScope(scope);
        setName(name);
    }
    public MethodCallExpr(final Expression scope, final String name, final List<Expression> args) {
        setScope(scope);
        setName(name);
        setArgs(args);
    }
    public MethodCallExpr(final Range range, final Expression scope, final List<Type<?>> typeArguments, final String name, final List<Expression> args) {
        super(range);
        setScope(scope);
        setTypeArguments(typeArguments);
        setName(name);
        setArgs(args);
    }
    public MethodCallExpr addArgument(Expression arg) {
        getArgs().add(arg);
        arg.setParentNode(this);
        return this;
    }
    public void addArgument(String arg) {
        addArgument(new NameExpr(arg));
    }
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<Expression> getArgs() {
        args = ensureNotNull(args);
        return args;
    }
    public String getName() {
        return name.getName();
    }
    public NameExpr getNameExpr() {
        return name;
    }
    public Expression getScope() {
        return scope;
    }
    public void setArgs(final List<Expression> args) {
        this.args = args;
        setAsParentNodeOf(this.args);
    }
    public MethodCallExpr setName(final String name) {
        setNameExpr(new NameExpr(name));
        return this;
    }
    public MethodCallExpr setNameExpr(NameExpr name) {
        this.name = name;
        setAsParentNodeOf(this.name);
        return this;
    }
    public MethodCallExpr setScope(final Expression scope) {
        this.scope = scope;
        setAsParentNodeOf(this.scope);
        return this;
    }
    @Override
    public List<Type<?>> getTypeArguments() {
        return typeArguments;
    }
    @Override
    public MethodCallExpr setTypeArguments(final List<Type<?>> types) {
        this.typeArguments = types;
        setAsParentNodeOf(this.typeArguments);
        return this;
    }
}
