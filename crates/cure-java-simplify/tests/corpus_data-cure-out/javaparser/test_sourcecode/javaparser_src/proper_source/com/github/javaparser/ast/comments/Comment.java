package com.github.javaparser.ast.comments;

import com.github.javaparser.ast.Node;

public abstract class Comment extends Node {
    private String content;
    private Node commentedNode;
    public Comment() {}
    public Comment(String content) {
        this.content = content;
    }
    public Comment(int beginLine, int beginColumn, int endLine, int endColumn, String content) {
        super(beginLine, beginColumn, endLine, endColumn);
        this.content = content;
    }
    public final String getContent() {
        return content;
    }
    public void setContent(String content) {
        this.content = content;
    }
    public boolean isLineComment() {
        return false;
    }
    public LineComment asLineComment() {
        if (isLineComment()) {
            return (LineComment) this;
        } else {
            throw new UnsupportedOperationException("Not a line comment");
        }
    }
    public Node getCommentedNode() {
        return this.commentedNode;
    }
    public void setCommentedNode(Node commentedNode) {
        if (commentedNode == null) {
            this.commentedNode = commentedNode;
            return;
        }
        if (commentedNode == this) {
            throw new IllegalArgumentException();
        }
        if (commentedNode instanceof Comment) {
            throw new IllegalArgumentException();
        }
        this.commentedNode = commentedNode;
    }
    public boolean isOrphan() {
        return this.commentedNode == null;
    }
}
