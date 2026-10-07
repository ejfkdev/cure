package com.github.javaparser.ast.body;

import static com.github.javaparser.utils.Utils.ensureNotNull;
import static com.github.javaparser.utils.Utils.isNullOrEmpty;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithJavaDoc;
import com.github.javaparser.ast.nodeTypes.NodeWithMembers;
import com.github.javaparser.ast.nodeTypes.NodeWithModifiers;
import com.github.javaparser.ast.nodeTypes.NodeWithName;

public abstract class TypeDeclaration<T> extends BodyDeclaration<T> implements NodeWithName<T>, NodeWithJavaDoc<T>, NodeWithModifiers<T>, NodeWithMembers<T> {
    private NameExpr name;
    private EnumSet<Modifier> modifiers = EnumSet.noneOf(Modifier.class);
    private List<BodyDeclaration<?>> members;
    public TypeDeclaration() {}
    public TypeDeclaration(EnumSet<Modifier> modifiers, String name) {
        setName(name);
        setModifiers(modifiers);
    }
    public TypeDeclaration(List<AnnotationExpr> annotations, EnumSet<Modifier> modifiers, String name, List<BodyDeclaration<?>> members) {
        super(annotations);
        setName(name);
        setModifiers(modifiers);
        setMembers(members);
    }
    public TypeDeclaration(Range range, List<AnnotationExpr> annotations, EnumSet<Modifier> modifiers, String name, List<BodyDeclaration<?>> members) {
        super(range, annotations);
        setName(name);
        setModifiers(modifiers);
        setMembers(members);
    }
    public TypeDeclaration<T> addMember(BodyDeclaration<?> decl) {
        List<BodyDeclaration<?>> members = getMembers();
        if (isNullOrEmpty(members)) {
            members = new ArrayList<>();
            setMembers(members);
        }
        members.add(decl);
        decl.setParentNode(this);
        return this;
    }
    @Override
    public List<BodyDeclaration<?>> getMembers() {
        members = ensureNotNull(members);
        return members;
    }
    @Override
    public final EnumSet<Modifier> getModifiers() {
        return modifiers;
    }
    @Override
	public final String getName() {
        return name.getName();
    }
    @SuppressWarnings("unchecked")
    @Override
    public T setMembers(List<BodyDeclaration<?>> members) {
        this.members = members;
        setAsParentNodeOf(this.members);
        return (T) this;
    }
    @SuppressWarnings("unchecked")
    @Override
    public T setModifiers(EnumSet<Modifier> modifiers) {
        this.modifiers = modifiers;
        return (T) this;
    }
    @Override
    @SuppressWarnings("unchecked")
    public T setName(String name) {
        setNameExpr(new NameExpr(name));
        return (T) this;
    }
    @SuppressWarnings("unchecked")
    public T setNameExpr(NameExpr nameExpr) {
        this.name = nameExpr;
        setAsParentNodeOf(this.name);
        return (T) this;
    }
    public final NameExpr getNameExpr() {
        return name;
    }
    @Override
	public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
}
