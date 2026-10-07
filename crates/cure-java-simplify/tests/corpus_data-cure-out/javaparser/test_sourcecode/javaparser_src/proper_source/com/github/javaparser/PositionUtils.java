package com.github.javaparser;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.AnnotableNode;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.expr.AnnotationExpr;
import java.lang.Override;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedList;
import java.util.List;
import static java.lang.Integer.signum;

public final class PositionUtils {
    private PositionUtils() {}
    public static <T extends Node> void sortByBeginPosition(List<T> nodes) {
        sortByBeginPosition(nodes, false);
    }
    public static <T extends Node> void sortByBeginPosition(List<T> nodes, final boolean ignoringAnnotations) {
        Collections.sort(nodes, new Comparator<Node>() {
            @Override
            public int compare(Node o1, Node o2) {
                return PositionUtils.compare(o1, o2, ignoringAnnotations);
            }
        });
    }
    public static boolean areInOrder(Node a, Node b) {
        return areInOrder(a, b, false);
    }
    public static boolean areInOrder(Node a, Node b, boolean ignoringAnnotations) {
        return compare(a, b, ignoringAnnotations) <= 0;
    }
    private static int compare(Node a, Node b, boolean ignoringAnnotations) {
        if (ignoringAnnotations) {
            int signLine = signum(beginLineWithoutConsideringAnnotation(a) - beginLineWithoutConsideringAnnotation(b));
            return signLine == 0 ? signum(beginColumnWithoutConsideringAnnotation(a) - beginColumnWithoutConsideringAnnotation(b)) : signLine;
        }
        int signLine = signum(a.getBeginLine() - b.getBeginLine());
        return signLine == 0 ? signum(a.getBeginColumn() - b.getBeginColumn()) : signLine;
    }
    public static AnnotationExpr getLastAnnotation(Node node) {
        if (node instanceof AnnotableNode) {
            List<AnnotationExpr> annotations = new LinkedList<AnnotationExpr>();
            annotations.addAll(((AnnotableNode) node).getAnnotations());
            if (annotations.size() == 0) {
                return null;
            }
            sortByBeginPosition(annotations);
            return annotations.get(annotations.size() - 1);
        } else {
            return null;
        }
    }
    private static int beginLineWithoutConsideringAnnotation(Node node) {
        return beginNodeWithoutConsideringAnnotations(node).getBeginLine();
    }
    private static int beginColumnWithoutConsideringAnnotation(Node node) {
        return beginNodeWithoutConsideringAnnotations(node).getBeginColumn();
    }
    private static Node beginNodeWithoutConsideringAnnotations(Node node) {
        return node instanceof MethodDeclaration ? ((MethodDeclaration) node).getType() : node instanceof FieldDeclaration ? ((FieldDeclaration) node).getType() : node instanceof ClassOrInterfaceDeclaration ? ((ClassOrInterfaceDeclaration) node).getNameExpr() : node;
    }
    public static boolean nodeContains(Node container, Node contained, boolean ignoringAnnotations) {
        if (!ignoringAnnotations || PositionUtils.getLastAnnotation(container) == null) {
            return container.contains(contained);
        }
        if (!container.contains(contained)) {
            return false;
        }
        if (container instanceof AnnotableNode) {
            int bl = beginLineWithoutConsideringAnnotation(container);
            int bc = beginColumnWithoutConsideringAnnotation(container);
            return bl > contained.getBeginLine() ? false : bl == contained.getBeginLine() && bc > contained.getBeginColumn() ? false : container.getEndLine() < contained.getEndLine() ? false : !(container.getEndLine() == contained.getEndLine() && container.getEndColumn() < contained.getEndColumn());
        }
        return true;
    }
}
