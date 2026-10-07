package com.github.javaparser.ast;

import static com.github.javaparser.ast.expr.NameExpr.*;
import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.Arrays;
import java.util.EnumSet;
import java.util.List;
import java.util.stream.Collectors;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.utils.ClassUtils;
import com.github.javaparser.Range;
import com.github.javaparser.ast.body.*;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import com.github.javaparser.utils.ClassUtils;
import java.util.Arrays;
import java.util.EnumSet;
import java.util.List;
import java.util.stream.Collectors;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class CompilationUnit extends Node {
    private PackageDeclaration pakage;
    private List<ImportDeclaration> imports;
    private List<TypeDeclaration<?>> types;
    public CompilationUnit() {}
    public CompilationUnit(PackageDeclaration pakage, List<ImportDeclaration> imports, List<TypeDeclaration<?>> types) {
        setPackage(pakage);
        setImports(imports);
        setTypes(types);
    }
    public CompilationUnit(Range range, PackageDeclaration pakage, List<ImportDeclaration> imports, List<TypeDeclaration<?>> types) {
        super(range);
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
        imports = ensureNotNull(imports);
        return imports;
    }
    public PackageDeclaration getPackage() {
        return pakage;
    }
    public List<TypeDeclaration<?>> getTypes() {
        types = ensureNotNull(types);
        return types;
    }
    public CompilationUnit setComments(List<Comment> comments) {
        throw new RuntimeException("Not implemented!");
    }
    public CompilationUnit setImports(List<ImportDeclaration> imports) {
        this.imports = imports;
        setAsParentNodeOf(this.imports);
        return this;
    }
    public CompilationUnit setPackage(PackageDeclaration pakage) {
        this.pakage = pakage;
        setAsParentNodeOf(this.pakage);
        return this;
    }
    public CompilationUnit setTypes(List<TypeDeclaration<?>> types) {
        this.types = types;
        setAsParentNodeOf(this.types);
        return this;
    }
    public CompilationUnit setPackageName(String name) {
        setPackage(new PackageDeclaration(name(name)));
        return this;
    }
    public CompilationUnit addImport(String name) {
        return addImport(name, false, false);
    }
    public CompilationUnit addImport(Class<?> clazz) {
        if (ClassUtils.isPrimitiveOrWrapper(clazz) || clazz.getName().startsWith("java.lang")) 
            return this; else if (clazz.isArray() && !ClassUtils.isPrimitiveOrWrapper(clazz.getComponentType()) && !clazz.getComponentType().getName().startsWith("java.lang")) 
            return addImport(clazz.getComponentType().getName());
        return addImport(clazz.getName());
    }
    public CompilationUnit addImport(String name, boolean isStatic, boolean isAsterisk) {
        if (getImports().stream().anyMatch((i) -> i.getName().toString().equals(name))) 
            return this; else {
            ImportDeclaration importDeclaration = new ImportDeclaration(name(name), isStatic, isAsterisk);
            getImports().add(importDeclaration);
            importDeclaration.setParentNode(this);
            return this;
        }
    }
    public ClassOrInterfaceDeclaration addClass(String name) {
        return addClass(name, Modifier.PUBLIC);
    }
    public ClassOrInterfaceDeclaration addClass(String name, Modifier... modifiers) {
        ClassOrInterfaceDeclaration classOrInterfaceDeclaration = new ClassOrInterfaceDeclaration(Arrays.stream(modifiers).collect(Collectors.toCollection(() -> EnumSet.noneOf(Modifier.class))), false, name);
        getTypes().add(classOrInterfaceDeclaration);
        classOrInterfaceDeclaration.setParentNode(this);
        return classOrInterfaceDeclaration;
    }
    public ClassOrInterfaceDeclaration addInterface(String name) {
        return addInterface(name, Modifier.PUBLIC);
    }
    public ClassOrInterfaceDeclaration addInterface(String name, Modifier... modifiers) {
        ClassOrInterfaceDeclaration classOrInterfaceDeclaration = new ClassOrInterfaceDeclaration(Arrays.stream(modifiers).collect(Collectors.toCollection(() -> EnumSet.noneOf(Modifier.class))), true, name);
        getTypes().add(classOrInterfaceDeclaration);
        classOrInterfaceDeclaration.setParentNode(this);
        return classOrInterfaceDeclaration;
    }
    public EnumDeclaration addEnum(String name) {
        return addEnum(name, Modifier.PUBLIC);
    }
    public EnumDeclaration addEnum(String name, Modifier... modifiers) {
        EnumDeclaration enumDeclaration = new EnumDeclaration(Arrays.stream(modifiers).collect(Collectors.toCollection(() -> EnumSet.noneOf(Modifier.class))), name);
        getTypes().add(enumDeclaration);
        enumDeclaration.setParentNode(this);
        return enumDeclaration;
    }
    public AnnotationDeclaration addAnnotationDeclaration(String name) {
        return addAnnotationDeclaration(name, Modifier.PUBLIC);
    }
    public AnnotationDeclaration addAnnotationDeclaration(String name, Modifier... modifiers) {
        AnnotationDeclaration annotationDeclaration = new AnnotationDeclaration(Arrays.stream(modifiers).collect(Collectors.toCollection(() -> EnumSet.noneOf(Modifier.class))), name);
        getTypes().add(annotationDeclaration);
        annotationDeclaration.setParentNode(this);
        return annotationDeclaration;
    }
    public ClassOrInterfaceDeclaration getClassByName(String className) {
        return (ClassOrInterfaceDeclaration) getTypes().stream().filter((type) -> type.getName().equals(className) && type instanceof ClassOrInterfaceDeclaration && !((ClassOrInterfaceDeclaration) type).isInterface()).findFirst().orElse(null);
    }
    public ClassOrInterfaceDeclaration getInterfaceByName(String interfaceName) {
        return (ClassOrInterfaceDeclaration) getTypes().stream().filter((type) -> type.getName().equals(interfaceName) && type instanceof ClassOrInterfaceDeclaration && ((ClassOrInterfaceDeclaration) type).isInterface()).findFirst().orElse(null);
    }
    public EnumDeclaration getEnumByName(String enumName) {
        return (EnumDeclaration) getTypes().stream().filter((type) -> type.getName().equals(enumName) && type instanceof EnumDeclaration).findFirst().orElse(null);
    }
    public AnnotationDeclaration getAnnotationDeclarationByName(String annotationName) {
        return (AnnotationDeclaration) getTypes().stream().filter((type) -> type.getName().equals(annotationName) && type instanceof AnnotationDeclaration).findFirst().orElse(null);
    }
}
