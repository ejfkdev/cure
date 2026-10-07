package com.github.javaparser.ast;

import com.github.javaparser.Position;
import com.github.javaparser.Range;
import com.github.javaparser.ast.comments.BlockComment;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.LineComment;
import com.github.javaparser.ast.visitor.*;
import java.util.*;

public abstract class Node implements Cloneable {
    public static Comparator<Node> NODE_BY_BEGIN_POSITION = (a, b) -> a.getBegin().compareTo(b.getBegin());
    private Range range;
    private Node parentNode;
    private List<Node> childrenNodes = new LinkedList<>();
    private List<Comment> orphanComments = new LinkedList<>();
    private IdentityHashMap<UserDataKey<?>, Object> userData = null;
    private Comment comment;
    public Node() {
        this(Range.UNKNOWN);
    }
    public Node(Range range) {
        this.range = range;
    }
    public abstract <R, A> R accept(GenericVisitor<R, A> v, A arg);
    public abstract <A> void accept(VoidVisitor<A> v, A arg);
    public final Comment getComment() {
        return comment;
    }
    public Position getBegin() {
        return range.begin;
    }
    public Position getEnd() {
        return range.end;
    }
    public Node setBegin(Position begin) {
        range = range.withBegin(begin);
        return this;
    }
    public Node setEnd(Position end) {
        range = range.withEnd(end);
        return this;
    }
    public Range getRange() {
        return range;
    }
    public Node setRange(Range range) {
        this.range = range;
        return this;
    }
    public final Node setComment(final Comment comment) {
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
        return this;
    }
    public final Node setLineComment(String comment) {
        return setComment(new LineComment(comment));
    }
    public final Node setBlockComment(String comment) {
        return setComment(new BlockComment(comment));
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
    @SuppressWarnings("unchecked")
    public <T> T getParentNodeOfType(Class<T> classType) {
        Node parent = parentNode;
        while (parent != null) {
            if (classType.isAssignableFrom(parent.getClass())) 
                return (T) parent;
            parent = parent.parentNode;
        }
        return null;
    }
    public List<Node> getChildrenNodes() {
        return childrenNodes;
    }
    public boolean contains(Node other) {
        return range.contains(other.range);
    }
    public void addOrphanComment(Comment comment) {
        orphanComments.add(comment);
        comment.setParentNode(this);
    }
    public List<Comment> getOrphanComments() {
        return orphanComments;
    }
    public List<Comment> getAllContainedComments() {
        List<Comment> comments = new LinkedList<>();
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
            for (Node current : childNodes) {
                current.setParentNode(this);
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
    public boolean isPositionedAfter(Position position) {
        return range.isAfter(position);
    }
    public boolean isPositionedBefore(Position position) {
        return range.isBefore(position);
    }
    public boolean hasComment() {
        return comment != null;
    }
    public void tryAddImportToParentCompilationUnit(Class<?> clazz) {
        CompilationUnit parentNode = getParentNodeOfType(CompilationUnit.class);
        if (parentNode != null) {
            parentNode.addImport(clazz);
        }
    }
    public <N extends Node> List<N> getNodesByType(Class<N> clazz) {
        List<N> nodes = new ArrayList<>();
        for (Node child : getChildrenNodes()) {
            if (clazz.isInstance(child)) {
                nodes.add(clazz.cast(child));
            }
            nodes.addAll(child.getNodesByType(clazz));
        }
        return nodes;
    }
    public <M> M getUserData(final UserDataKey<M> key) {
        return userData == null ? null : (M) userData.get(key);
    }
    public <M> void setUserData(UserDataKey<M> key, M object) {
        if (userData == null) {
            userData = new IdentityHashMap<>();
        }
        userData.put(key, object);
    }
}
