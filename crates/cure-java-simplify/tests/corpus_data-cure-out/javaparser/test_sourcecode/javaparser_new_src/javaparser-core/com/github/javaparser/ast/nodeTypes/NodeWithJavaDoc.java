package com.github.javaparser.ast.nodeTypes;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.comments.JavadocComment;

public interface NodeWithJavaDoc<T> {
    JavadocComment getJavaDoc();
    @SuppressWarnings("unchecked")
    public default T setJavaDocComment(String comment) {
        ((Node) this).setComment(new JavadocComment(comment));
        return (T) this;
    }
}
