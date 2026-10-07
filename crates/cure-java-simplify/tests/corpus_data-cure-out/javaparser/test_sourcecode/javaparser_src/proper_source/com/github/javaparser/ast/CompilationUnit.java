package com.github.javaparser.ast;

import com.github.javaparser.ast.body.AnnotationDeclaration;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.EmptyTypeDeclaration;
import com.github.javaparser.ast.body.EnumDeclaration;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.body.TypeDeclaration;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;

public final class CompilationUnit extends Node {
    private PackageDeclaration pakage;
    private List<ImportDeclaration> imports;
    private List<TypeDeclaration> types;
    public CompilationUnit() {}
    public CompilationUnit(PackageDeclaration pakage, List<ImportDeclaration> imports, List<TypeDeclaration> types) {
        setPackage(pakage);
        setImports(imports);
        setTypes(types);
    }
    public CompilationUnit(int beginLine, int beginColumn, int endLine, int endColumn, PackageDeclaration pakage, List<ImportDeclaration> imports, List<TypeDeclaration> types) {
        super(beginLine, beginColumn, endLine, endColumn);
        setPackage(pakage);
        setImports(imports);
        setTypes(types);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public List<Comment> getComments() {
        return this.getAllContainedComments();
    }
    public List<ImportDeclaration> getImports() {
        return imports;
    }
    public PackageDeclaration getPackage() {
        return pakage;
    }
    public List<TypeDeclaration> getTypes() {
        return types;
    }
    public void setComments(List<Comment> comments) {
        throw new RuntimeException("Not implemented!");
    }
    public void setImports(List<ImportDeclaration> imports) {
        this.imports = imports;
        setAsParentNodeOf(this.imports);
    }
    public void setPackage(PackageDeclaration pakage) {
        this.pakage = pakage;
        setAsParentNodeOf(this.pakage);
    }
    public void setTypes(List<TypeDeclaration> types) {
        this.types = types;
        setAsParentNodeOf(this.types);
    }
}
