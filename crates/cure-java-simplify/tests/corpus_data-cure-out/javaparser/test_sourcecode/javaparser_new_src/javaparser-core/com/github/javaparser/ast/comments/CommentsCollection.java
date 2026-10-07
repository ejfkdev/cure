package com.github.javaparser.ast.comments;

import java.util.Collection;
import java.util.Set;
import java.util.TreeSet;
import java.util.stream.Collectors;
import static com.github.javaparser.ast.Node.NODE_BY_BEGIN_POSITION;

public class CommentsCollection {
    private final TreeSet<Comment> comments = new TreeSet<>(NODE_BY_BEGIN_POSITION);
    public CommentsCollection() {}
    public CommentsCollection(Collection<Comment> commentsToCopy) {
        comments.addAll(commentsToCopy);
    }
    public Set<LineComment> getLineComments() {
        return comments.stream().filter((comment) -> comment instanceof LineComment).map((comment) -> (LineComment) comment).collect(Collectors.toCollection(() -> new TreeSet<>(NODE_BY_BEGIN_POSITION)));
    }
    public Set<BlockComment> getBlockComments() {
        return comments.stream().filter((comment) -> comment instanceof BlockComment).map((comment) -> (BlockComment) comment).collect(Collectors.toCollection(() -> new TreeSet<>(NODE_BY_BEGIN_POSITION)));
    }
    public Set<JavadocComment> getJavadocComments() {
        return comments.stream().filter((comment) -> comment instanceof JavadocComment).map((comment) -> (JavadocComment) comment).collect(Collectors.toCollection(() -> new TreeSet<>(NODE_BY_BEGIN_POSITION)));
    }
    public void addComment(Comment comment) {
        comments.add(comment);
    }
    public boolean contains(Comment comment) {
        for (Comment c : getComments()) {
            if (c.getBegin().line == comment.getBegin().line && c.getBegin().column == comment.getBegin().column && c.getEnd().line == comment.getEnd().line && Math.abs(c.getEnd().column - comment.getEnd().column) < 2) {
                return true;
            }
        }
        return false;
    }
    public TreeSet<Comment> getComments() {
        return comments;
    }
    public int size() {
        return comments.size();
    }
    public CommentsCollection minus(CommentsCollection other) {
        CommentsCollection result = new CommentsCollection();
        result.comments.addAll(comments.stream().filter((comment) -> !other.contains(comment)).collect(Collectors.toList()));
        return result;
    }
    public CommentsCollection copy() {
        return new CommentsCollection(comments);
    }
}
