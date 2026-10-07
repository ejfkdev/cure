package com.github.javaparser.ast.body;

import java.util.List;
import java.util.EnumSet;
import com.github.javaparser.Range;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class AnnotationDeclaration extends TypeDeclaration<AnnotationDeclaration> {
    public AnnotationDeclaration() {}
    public AnnotationDeclaration(EnumSet<Modifier> modifiers, String name) {
        super(modifiers, name);
    }
    public AnnotationDeclaration(EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, String name, List<BodyDeclaration<?>> members) {
        super(annotations, modifiers, name, members);
    }
    public AnnotationDeclaration(Range range, EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, String name, List<BodyDeclaration<?>> members) {
        super(range, annotations, modifiers, name, members);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
}
