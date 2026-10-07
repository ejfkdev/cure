package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.type.TypeParameter;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithExtends;
import com.github.javaparser.ast.nodeTypes.NodeWithImplements;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.EnumSet;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class ClassOrInterfaceDeclaration extends TypeDeclaration<ClassOrInterfaceDeclaration> implements NodeWithImplements<ClassOrInterfaceDeclaration>, NodeWithExtends<ClassOrInterfaceDeclaration> {
    private boolean interface_;
    private List<TypeParameter> typeParameters;
    private List<ClassOrInterfaceType> extendsList;
    private List<ClassOrInterfaceType> implementsList;
    public ClassOrInterfaceDeclaration() {}
    public ClassOrInterfaceDeclaration(final EnumSet<Modifier> modifiers, final boolean isInterface, final String name) {
        super(modifiers, name);
        setInterface(isInterface);
    }
    public ClassOrInterfaceDeclaration(final EnumSet<Modifier> modifiers, final List<AnnotationExpr> annotations, final boolean isInterface, final String name, final List<TypeParameter> typeParameters, final List<ClassOrInterfaceType> extendsList, final List<ClassOrInterfaceType> implementsList, final List<BodyDeclaration<?>> members) {
        super(annotations, modifiers, name, members);
        setInterface(isInterface);
        setTypeParameters(typeParameters);
        setExtends(extendsList);
        setImplements(implementsList);
    }
    public ClassOrInterfaceDeclaration(Range range, final EnumSet<Modifier> modifiers, final List<AnnotationExpr> annotations, final boolean isInterface, final String name, final List<TypeParameter> typeParameters, final List<ClassOrInterfaceType> extendsList, final List<ClassOrInterfaceType> implementsList, final List<BodyDeclaration<?>> members) {
        super(range, annotations, modifiers, name, members);
        setInterface(isInterface);
        setTypeParameters(typeParameters);
        setExtends(extendsList);
        setImplements(implementsList);
    }
    @Override
    public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public List<ClassOrInterfaceType> getExtends() {
        extendsList = ensureNotNull(extendsList);
        return extendsList;
    }
    @Override
    public List<ClassOrInterfaceType> getImplements() {
        implementsList = ensureNotNull(implementsList);
        return implementsList;
    }
    public List<TypeParameter> getTypeParameters() {
        typeParameters = ensureNotNull(typeParameters);
        return typeParameters;
    }
    public boolean isInterface() {
        return interface_;
    }
    @Override
    public ClassOrInterfaceDeclaration setExtends(final List<ClassOrInterfaceType> extendsList) {
        this.extendsList = extendsList;
        setAsParentNodeOf(this.extendsList);
        return this;
    }
    @Override
    public ClassOrInterfaceDeclaration setImplements(final List<ClassOrInterfaceType> implementsList) {
        this.implementsList = implementsList;
        setAsParentNodeOf(this.implementsList);
        return this;
    }
    public ClassOrInterfaceDeclaration setInterface(final boolean interface_) {
        this.interface_ = interface_;
        return this;
    }
    public ClassOrInterfaceDeclaration setTypeParameters(final List<TypeParameter> typeParameters) {
        this.typeParameters = typeParameters;
        setAsParentNodeOf(this.typeParameters);
        return this;
    }
}
