package com.github.javaparser.ast.comments;

import com.github.javaparser.Range;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class LineComment extends Comment {
    public LineComment() {}
    public LineComment(String content) {
        super(content);
    }
    public LineComment(Range range, String content) {
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
    @Override
    public boolean isLineComment() {
        return true;
    }
}
