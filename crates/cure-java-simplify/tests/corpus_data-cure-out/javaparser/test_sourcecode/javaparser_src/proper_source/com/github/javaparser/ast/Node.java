package com.github.javaparser.ast;

import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.visitor.*;

public abstract class Node implements Cloneable {
    private int beginLine;
    private int beginColumn;
    private int endLine;
    private int endColumn;
    private Node parentNode;
    private List<Node> childrenNodes = new LinkedList<Node>();
    private List<Comment> orphanComments = new LinkedList<Comment>();
    private Object data;
    private Comment comment;
    public Node() {}
    public Node(final int beginLine, final int beginColumn, final int endLine, final int endColumn) {
        this.beginLine = beginLine;
        this.beginColumn = beginColumn;
        this.endLine = endLine;
        this.endColumn = endColumn;
    }
    public abstract <R, A> R accept(GenericVisitor<R, A> v, A arg);
    public abstract <A> void accept(VoidVisitor<A> v, A arg);
    public final int getBeginColumn() {
        return beginColumn;
    }
    public final int getBeginLine() {
        return beginLine;
    }
    public final Comment getComment() {
        return comment;
    }
    public final Object getData() {
        return data;
    }
    public final int getEndColumn() {
        return endColumn;
    }
    public final int getEndLine() {
        return endLine;
    }
    public final void setBeginColumn(final int beginColumn) {
        this.beginColumn = beginColumn;
    }
    public final void setBeginLine(final int beginLine) {
        this.beginLine = beginLine;
    }
    public final void setComment(final Comment comment) {
        if (comment != null && this instanceof Comment) {
            throw new RuntimeException("A comment can not be commented");
        }
        if (this.comment != null) {
            this.comment.setCommentedNode(null);
        }
        this.comment = comment;
        if (comment != null) {
            this.comment.setCommentedNode(this);
        }
    }
    public final void setData(final Object data) {
        this.data = data;
    }
    public final void setEndColumn(final int endColumn) {
        this.endColumn = endColumn;
    }
    public final void setEndLine(final int endLine) {
        this.endLine = endLine;
    }
    @Override
    public final String toString() {
        DumpVisitor visitor = new DumpVisitor();
        accept(visitor, null);
        return visitor.getSource();
    }
    public final String toStringWithoutComments() {
        DumpVisitor visitor = new DumpVisitor(false);
        accept(visitor, null);
        return visitor.getSource();
    }
    @Override
    public final int hashCode() {
        return toString().hashCode();
    }
    @Override
    public boolean equals(final Object obj) {
        return obj == null || !(obj instanceof Node) ? false : EqualsVisitor.equals(this, (Node) obj);
    }
    @Override
    public Node clone() {
        return this.accept(new CloneVisitor(), null);
    }
    public Node getParentNode() {
        return parentNode;
    }
    public List<Node> getChildrenNodes() {
        return childrenNodes;
    }
    public boolean contains(Node other) {
        return getBeginLine() > other.getBeginLine() ? false : getBeginLine() == other.getBeginLine() && getBeginColumn() > other.getBeginColumn() ? false : getEndLine() < other.getEndLine() ? false : !(getEndLine() == other.getEndLine() && getEndColumn() < other.getEndColumn());
    }
    public void addOrphanComment(Comment comment) {
        orphanComments.add(comment);
        comment.setParentNode(this);
    }
    public List<Comment> getOrphanComments() {
        return orphanComments;
    }
    public List<Comment> getAllContainedComments() {
        List<Comment> comments = new LinkedList<Comment>();
        comments.addAll(getOrphanComments());
        for (Node child : getChildrenNodes()) {
            if (child.getComment() != null) {
                comments.add(child.getComment());
            }
            comments.addAll(child.getAllContainedComments());
        }
        return comments;
    }
    public void setParentNode(Node parentNode) {
        if (this.parentNode != null) {
            this.parentNode.childrenNodes.remove(this);
        }
        this.parentNode = parentNode;
        if (this.parentNode != null) {
            this.parentNode.childrenNodes.add(this);
        }
    }
    protected void setAsParentNodeOf(List<? extends Node> childNodes) {
        if (childNodes != null) {
            for (Object e : childNodes) {
                e.setParentNode(this);
            }
        }
    }
    protected void setAsParentNodeOf(Node childNode) {
        if (childNode != null) {
            childNode.setParentNode(this);
        }
    }
    public static final int ABSOLUTE_BEGIN_LINE = -1;
    public static final int ABSOLUTE_END_LINE = -2;
    public boolean isPositionedAfter(int line, int column) {
        return line == ABSOLUTE_BEGIN_LINE || (getBeginLine() > line || getBeginLine() == line && getBeginColumn() > column);
    }
    public boolean isPositionedBefore(int line, int column) {
        return line == ABSOLUTE_END_LINE || (getEndLine() < line || getEndLine() == line && getEndColumn() < column);
    }
    public boolean hasComment() {
        return comment != null;
    }
}
