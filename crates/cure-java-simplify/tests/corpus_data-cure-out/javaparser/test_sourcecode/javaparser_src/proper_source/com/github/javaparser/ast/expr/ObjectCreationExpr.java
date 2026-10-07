package com.github.javaparser.ast.expr;

import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class ObjectCreationExpr extends Expression {
    private Expression scope;
    private ClassOrInterfaceType type;
    private List<Type> typeArgs;
    private List<Expression> args;
    private List<BodyDeclaration> anonymousClassBody;
    public ObjectCreationExpr() {}
    public ObjectCreationExpr(final Expression scope, final ClassOrInterfaceType type, final List<Expression> args) {
        setScope(scope);
        setType(type);
        setArgs(args);
    }
    public ObjectCreationExpr(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final Expression scope, final ClassOrInterfaceType type, final List<Type> typeArgs, final List<Expression> args, final List<BodyDeclaration> anonymousBody) {
        super(beginLine, beginColumn, endLine, endColumn);
        setScope(scope);
        setType(type);
        setTypeArgs(typeArgs);
        setArgs(args);
        setAnonymousClassBody(anonymousBody);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<BodyDeclaration> getAnonymousClassBody() {
        return anonymousClassBody;
    }
    public List<Expression> getArgs() {
        return args;
    }
    public Expression getScope() {
        return scope;
    }
    public ClassOrInterfaceType getType() {
        return type;
    }
    public List<Type> getTypeArgs() {
        return typeArgs;
    }
    public void setAnonymousClassBody(final List<BodyDeclaration> anonymousClassBody) {
        this.anonymousClassBody = anonymousClassBody;
        setAsParentNodeOf(this.anonymousClassBody);
    }
    public void setArgs(final List<Expression> args) {
        this.args = args;
        setAsParentNodeOf(this.args);
    }
    public void setScope(final Expression scope) {
        this.scope = scope;
        setAsParentNodeOf(this.scope);
    }
    public void setType(final ClassOrInterfaceType type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
    public void setTypeArgs(final List<Type> typeArgs) {
        this.typeArgs = typeArgs;
        setAsParentNodeOf(this.typeArgs);
    }
}
