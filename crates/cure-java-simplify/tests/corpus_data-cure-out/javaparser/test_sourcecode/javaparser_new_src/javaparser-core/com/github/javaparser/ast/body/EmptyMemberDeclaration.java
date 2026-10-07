package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.nodeTypes.NodeWithJavaDoc;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class EmptyMemberDeclaration extends BodyDeclaration<EmptyMemberDeclaration> implements NodeWithJavaDoc<EmptyMemberDeclaration> {
    public EmptyMemberDeclaration() {
        super(null);
    }
    public EmptyMemberDeclaration(Range range) {
        super(range, null);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    @Override
    public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
}
