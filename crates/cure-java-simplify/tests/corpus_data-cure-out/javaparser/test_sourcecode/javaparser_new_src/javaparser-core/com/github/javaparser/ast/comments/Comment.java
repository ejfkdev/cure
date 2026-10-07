package com.github.javaparser.ast.comments;

import com.github.javaparser.Range;
import com.github.javaparser.ast.Node;

public abstract class Comment extends Node {
    private String content;
    private Node commentedNode;
    public Comment() {}
    public Comment(String content) {
        this.content = content;
    }
    public Comment(Range range, String content) {
        super(range);
        this.content = content;
    }
    public final String getContent() {
        return content;
    }
    public Comment setContent(String content) {
        this.content = content;
        return this;
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
    public Comment setCommentedNode(Node commentedNode) {
        if (commentedNode == null) {
            this.commentedNode = null;
            return this;
        }
        if (commentedNode == this) {
            throw new IllegalArgumentException();
        }
        if (commentedNode instanceof Comment) {
            throw new IllegalArgumentException();
        }
        this.commentedNode = commentedNode;
        return this;
    }
    public boolean isOrphan() {
        return this.commentedNode == null;
    }
}
