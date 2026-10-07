package com.github.javaparser.ast.comments;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class JavadocComment extends Comment {
    public JavadocComment() {}
    public JavadocComment(String content) {
        super(content);
    }
    public JavadocComment(Range range, String content) {
        super(range, content);
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
