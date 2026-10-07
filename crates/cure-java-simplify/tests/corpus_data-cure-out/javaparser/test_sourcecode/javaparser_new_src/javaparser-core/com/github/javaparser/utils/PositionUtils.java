package com.github.javaparser.utils;

import static java.lang.Integer.signum;
import java.util.Collections;
import java.util.LinkedList;
import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.ast.nodeTypes.NodeWithElementType;
import com.github.javaparser.ast.nodeTypes.NodeWithType;

public final class PositionUtils {
    private PositionUtils() {}
    public static <T extends Node> void sortByBeginPosition(List<T> nodes) {
        sortByBeginPosition(nodes, false);
    }
    public static <T extends Node> void sortByBeginPosition(List<T> nodes, final boolean ignoringAnnotations) {
        Collections.sort(nodes, (o1, o2) -> PositionUtils.compare(o1, o2, ignoringAnnotations));
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
        int signLine = signum(a.getBegin().line - b.getBegin().line);
        return signLine == 0 ? signum(a.getBegin().column - b.getBegin().column) : signLine;
    }
    public static AnnotationExpr getLastAnnotation(Node node) {
        if (node instanceof NodeWithAnnotations) {
            List<AnnotationExpr> annotations = new LinkedList<>();
            annotations.addAll(((NodeWithAnnotations<?>) node).getAnnotations());
            if (annotations.isEmpty()) {
                return null;
            }
            sortByBeginPosition(annotations);
            return annotations.get(annotations.size() - 1);
        } else {
            return null;
        }
    }
    private static int beginLineWithoutConsideringAnnotation(Node node) {
        return beginNodeWithoutConsideringAnnotations(node).getBegin().line;
    }
    private static int beginColumnWithoutConsideringAnnotation(Node node) {
        return beginNodeWithoutConsideringAnnotations(node).getBegin().column;
    }
    private static Node beginNodeWithoutConsideringAnnotations(Node node) {
        return node instanceof MethodDeclaration || node instanceof FieldDeclaration ? ((NodeWithElementType<?>) node).getElementType() : node instanceof ClassOrInterfaceDeclaration ? ((ClassOrInterfaceDeclaration) node).getNameExpr() : node;
    }
    public static boolean nodeContains(Node container, Node contained, boolean ignoringAnnotations) {
        if (!ignoringAnnotations || PositionUtils.getLastAnnotation(container) == null) {
            return container.contains(contained);
        }
        if (!container.contains(contained)) {
            return false;
        }
        if (container instanceof NodeWithAnnotations) {
            int bl = beginLineWithoutConsideringAnnotation(container);
            int bc = beginColumnWithoutConsideringAnnotation(container);
            return bl > contained.getBegin().line ? false : bl == contained.getBegin().line && bc > contained.getBegin().column ? false : container.getEnd().line < contained.getEnd().line ? false : !(container.getEnd().line == contained.getEnd().line && container.getEnd().column < contained.getEnd().column);
        }
        return true;
    }
}
