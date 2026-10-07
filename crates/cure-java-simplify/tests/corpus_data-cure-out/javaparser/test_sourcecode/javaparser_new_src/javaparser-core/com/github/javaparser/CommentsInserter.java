package com.github.javaparser;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.LineComment;
import com.github.javaparser.utils.PositionUtils;
import java.util.*;
import static com.github.javaparser.ast.Node.NODE_BY_BEGIN_POSITION;

class CommentsInserter {
    private final ParserConfiguration configuration;
    CommentsInserter(ParserConfiguration configuration) {
        this.configuration = configuration;
    }
    private void insertComments(CompilationUnit cu, TreeSet<Comment> comments) {
        if (comments.isEmpty()) 
            return;
        List<Node> children = cu.getChildrenNodes();
        PositionUtils.sortByBeginPosition(children);
        Comment firstComment = comments.iterator().next();
        if (cu.getPackage() != null && (children.isEmpty() || PositionUtils.areInOrder(firstComment, children.get(0)))) {
            cu.setComment(firstComment);
            comments.remove(firstComment);
        }
    }
    void insertComments(Node node, TreeSet<Comment> commentsToAttribute) {
        if (commentsToAttribute.isEmpty()) 
            return;
        if (node instanceof CompilationUnit) {
            insertComments((CompilationUnit) node, commentsToAttribute);
        }
        List<Node> children = node.getChildrenNodes();
        PositionUtils.sortByBeginPosition(children);
        for (Node child : children) {
            TreeSet<Comment> commentsInsideChild = new TreeSet<>(NODE_BY_BEGIN_POSITION);
            for (Comment c : commentsToAttribute) {
                if (PositionUtils.nodeContains(child, c, configuration.doNotConsiderAnnotationsAsNodeStartForCodeAttribution)) {
                    commentsInsideChild.add(c);
                }
            }
            commentsToAttribute.removeAll(commentsInsideChild);
            insertComments(child, commentsInsideChild);
        }
        List<Comment> attributedComments = new LinkedList<>();
        for (Comment comment : commentsToAttribute) {
            if (comment.isLineComment()) {
                for (Node child : children) {
                    if (child.getEnd().line == comment.getBegin().line && attributeLineCommentToNodeOrChild(child, comment.asLineComment())) {
                        attributedComments.add(comment);
                    }
                }
            }
        }
        Comment previousComment = null;
        attributedComments = new LinkedList<>();
        List<Node> childrenAndComments = new LinkedList<>();
        childrenAndComments.addAll(children);
        childrenAndComments.addAll(commentsToAttribute);
        PositionUtils.sortByBeginPosition(childrenAndComments, configuration.doNotConsiderAnnotationsAsNodeStartForCodeAttribution);
        for (Node thing : childrenAndComments) {
            if (thing instanceof Comment) {
                previousComment = (Comment) thing;
                if (!previousComment.isOrphan()) {
                    previousComment = null;
                }
            } else {
                if (previousComment != null && !thing.hasComment()) {
                    if (!configuration.doNotAssignCommentsPrecedingEmptyLines || !thereAreLinesBetween(previousComment, thing)) {
                        thing.setComment(previousComment);
                        attributedComments.add(previousComment);
                        previousComment = null;
                    }
                }
            }
        }
        commentsToAttribute.removeAll(attributedComments);
        for (Comment c : commentsToAttribute) {
            if (c.isOrphan()) {
                node.addOrphanComment(c);
            }
        }
    }
    private boolean attributeLineCommentToNodeOrChild(Node node, LineComment lineComment) {
        if (node.getBegin().line == lineComment.getBegin().line && !node.hasComment()) {
            if (!(node instanceof Comment)) {
                node.setComment(lineComment);
            }
            return true;
        } else {
            List<Node> children = new LinkedList<Node>();
            children.addAll(node.getChildrenNodes());
            PositionUtils.sortByBeginPosition(children);
            Collections.reverse(children);
            for (Node child : children) {
                if (attributeLineCommentToNodeOrChild(child, lineComment)) {
                    return true;
                }
            }
            return false;
        }
    }
    private boolean thereAreLinesBetween(Node a, Node b) {
        if (!PositionUtils.areInOrder(a, b)) {
            return thereAreLinesBetween(b, a);
        }
        int endOfA = a.getEnd().line;
        return b.getBegin().line > endOfA + 1;
    }
}
