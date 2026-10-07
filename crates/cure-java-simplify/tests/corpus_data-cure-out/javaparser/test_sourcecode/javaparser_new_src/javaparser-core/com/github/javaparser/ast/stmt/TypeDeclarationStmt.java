package com.github.javaparser.ast.stmt;

import com.github.javaparser.Range;
import com.github.javaparser.ast.body.TypeDeclaration;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class TypeDeclarationStmt extends Statement {
    private TypeDeclaration typeDecl;
    public TypeDeclarationStmt() {}
    public TypeDeclarationStmt(final TypeDeclaration typeDecl) {
        setTypeDeclaration(typeDecl);
    }
    public TypeDeclarationStmt(Range range, final TypeDeclaration typeDecl) {
        super(range);
        setTypeDeclaration(typeDecl);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public TypeDeclaration getTypeDeclaration() {
        return typeDecl;
    }
    public TypeDeclarationStmt setTypeDeclaration(final TypeDeclaration typeDecl) {
        this.typeDecl = typeDecl;
        setAsParentNodeOf(this.typeDecl);
        return this;
    }
}
