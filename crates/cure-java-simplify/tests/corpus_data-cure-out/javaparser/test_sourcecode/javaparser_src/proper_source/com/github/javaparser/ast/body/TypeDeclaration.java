package com.github.javaparser.ast.body;

import com.github.javaparser.ast.NamedNode;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.NameExpr;
import java.util.List;

public abstract class TypeDeclaration extends BodyDeclaration implements NamedNode {
    private NameExpr name;
    private int modifiers;
    private List<BodyDeclaration> members;
    public TypeDeclaration() {}
    public TypeDeclaration(int modifiers, String name) {
        setName(name);
        setModifiers(modifiers);
    }
    public TypeDeclaration(List<AnnotationExpr> annotations, int modifiers, String name, List<BodyDeclaration> members) {
        super(annotations);
        setName(name);
        setModifiers(modifiers);
        setMembers(members);
    }
    public TypeDeclaration(int beginLine, int beginColumn, int endLine, int endColumn, List<AnnotationExpr> annotations, int modifiers, String name, List<BodyDeclaration> members) {
        super(beginLine, beginColumn, endLine, endColumn, annotations);
        setName(name);
        setModifiers(modifiers);
        setMembers(members);
    }
    public final List<BodyDeclaration> getMembers() {
        return members;
    }
    public final int getModifiers() {
        return modifiers;
    }
    public final String getName() {
        return name.getName();
    }
    public void setMembers(List<BodyDeclaration> members) {
        this.members = members;
        setAsParentNodeOf(this.members);
    }
    public final void setModifiers(int modifiers) {
        this.modifiers = modifiers;
    }
    public final void setName(String name) {
        this.name = new NameExpr(name);
    }
    public final void setNameExpr(NameExpr nameExpr) {
        this.name = nameExpr;
    }
    public final NameExpr getNameExpr() {
        return name;
    }
}
