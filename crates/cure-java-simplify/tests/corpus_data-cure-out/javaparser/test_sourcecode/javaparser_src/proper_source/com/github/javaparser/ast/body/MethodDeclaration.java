package com.github.javaparser.ast.body;

import com.github.javaparser.ast.AccessSpecifier;
import com.github.javaparser.ast.DocumentableNode;
import com.github.javaparser.ast.NamedNode;
import com.github.javaparser.ast.TypeParameter;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import java.util.ArrayList;

public final class MethodDeclaration extends BodyDeclaration implements DocumentableNode, WithDeclaration, NamedNode {
    private int modifiers;
    private List<TypeParameter> typeParameters;
    private Type type;
    private NameExpr name;
    private List<Parameter> parameters;
    private int arrayCount;
    private List<NameExpr> throws_;
    private BlockStmt body;
    private boolean isDefault = false;
    public MethodDeclaration() {}
    public MethodDeclaration(final int modifiers, final Type type, final String name) {
        setModifiers(modifiers);
        setType(type);
        setName(name);
    }
    public MethodDeclaration(final int modifiers, final Type type, final String name, final List<Parameter> parameters) {
        setModifiers(modifiers);
        setType(type);
        setName(name);
        setParameters(parameters);
    }
    public MethodDeclaration(final int modifiers, final List<AnnotationExpr> annotations, final List<TypeParameter> typeParameters, final Type type, final String name, final List<Parameter> parameters, final int arrayCount, final List<NameExpr> throws_, final BlockStmt block) {
        super(annotations);
        setModifiers(modifiers);
        setTypeParameters(typeParameters);
        setType(type);
        setName(name);
        setParameters(parameters);
        setArrayCount(arrayCount);
        setThrows(throws_);
        setBody(block);
    }
    public MethodDeclaration(final int beginLine, final int beginColumn, final int endLine, final int endColumn, final int modifiers, final List<AnnotationExpr> annotations, final List<TypeParameter> typeParameters, final Type type, final String name, final List<Parameter> parameters, final int arrayCount, final List<NameExpr> throws_, final BlockStmt block) {
        super(beginLine, beginColumn, endLine, endColumn, annotations);
        setModifiers(modifiers);
        setTypeParameters(typeParameters);
        setType(type);
        setName(name);
        setParameters(parameters);
        setArrayCount(arrayCount);
        setThrows(throws_);
        setBody(block);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public int getArrayCount() {
        return arrayCount;
    }
    public BlockStmt getBody() {
        return body;
    }
    public int getModifiers() {
        return modifiers;
    }
    public String getName() {
        return name.getName();
    }
    public NameExpr getNameExpr() {
        return name;
    }
    public List<Parameter> getParameters() {
        if (parameters == null) {
            parameters = new ArrayList<Parameter>();
        }
        return parameters;
    }
    public List<NameExpr> getThrows() {
        if (throws_ == null) {
            throws_ = new ArrayList<NameExpr>();
        }
        return throws_;
    }
    public Type getType() {
        return type;
    }
    public List<TypeParameter> getTypeParameters() {
        return typeParameters;
    }
    public void setArrayCount(final int arrayCount) {
        this.arrayCount = arrayCount;
    }
    public void setBody(final BlockStmt body) {
        this.body = body;
        setAsParentNodeOf(this.body);
    }
    public void setModifiers(final int modifiers) {
        this.modifiers = modifiers;
    }
    public void setName(final String name) {
        this.name = new NameExpr(name);
    }
    public void setNameExpr(final NameExpr name) {
        this.name = name;
    }
    public void setParameters(final List<Parameter> parameters) {
        this.parameters = parameters;
        setAsParentNodeOf(this.parameters);
    }
    public void setThrows(final List<NameExpr> throws_) {
        this.throws_ = throws_;
        setAsParentNodeOf(this.throws_);
    }
    public void setType(final Type type) {
        this.type = type;
        setAsParentNodeOf(this.type);
    }
    public void setTypeParameters(final List<TypeParameter> typeParameters) {
        this.typeParameters = typeParameters;
        setAsParentNodeOf(typeParameters);
    }
    public boolean isDefault() {
        return isDefault;
    }
    public void setDefault(boolean isDefault) {
        this.isDefault = isDefault;
    }
    @Override
    public String getDeclarationAsString() {
        return getDeclarationAsString(true, true, true);
    }
    @Override
    public String getDeclarationAsString(boolean includingModifiers, boolean includingThrows) {
        return getDeclarationAsString(includingModifiers, includingThrows, true);
    }
    @Override
    public String getDeclarationAsString(boolean includingModifiers, boolean includingThrows, boolean includingParameterName) {
        StringBuffer sb = new StringBuffer();
        if (includingModifiers) {
            AccessSpecifier accessSpecifier = ModifierSet.getAccessSpecifier(getModifiers());
            sb.append(accessSpecifier.getCodeRepresenation());
            sb.append(accessSpecifier == AccessSpecifier.DEFAULT ? "" : " ");
            if (ModifierSet.isStatic(getModifiers())) {
                sb.append("static ");
            }
            if (ModifierSet.isAbstract(getModifiers())) {
                sb.append("abstract ");
            }
            if (ModifierSet.isFinal(getModifiers())) {
                sb.append("final ");
            }
            if (ModifierSet.isNative(getModifiers())) {
                sb.append("native ");
            }
            if (ModifierSet.isSynchronized(getModifiers())) {
                sb.append("synchronized ");
            }
        }
        sb.append(getType().toStringWithoutComments());
        sb.append(" ");
        sb.append(getName());
        sb.append("(");
        boolean firstParam = true;
        for (Parameter param : getParameters()) {
            if (firstParam) {
                firstParam = false;
            } else {
                sb.append(", ");
            }
            if (includingParameterName) {
                sb.append(param.toStringWithoutComments());
            } else {
                sb.append(param.getType().toStringWithoutComments());
                if (param.isVarArgs()) {
                    sb.append("...");
                }
            }
        }
        sb.append(")");
        if (includingThrows) {
            boolean firstThrow = true;
            for (NameExpr thr : getThrows()) {
                if (firstThrow) {
                    firstThrow = false;
                    sb.append(" throws ");
                } else {
                    sb.append(", ");
                }
                sb.append(thr.toStringWithoutComments());
            }
        }
        return sb.toString();
    }
    @Override
    public void setJavaDoc(JavadocComment javadocComment) {
        this.javadocComment = javadocComment;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return javadocComment;
    }
    private JavadocComment javadocComment;
}
