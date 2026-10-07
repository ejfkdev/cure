package com.github.javaparser.ast;

import com.github.javaparser.ast.comments.JavadocComment;

public interface DocumentableNode {
    public JavadocComment getJavaDoc();
    public void setJavaDoc(JavadocComment javadocComment);
}
