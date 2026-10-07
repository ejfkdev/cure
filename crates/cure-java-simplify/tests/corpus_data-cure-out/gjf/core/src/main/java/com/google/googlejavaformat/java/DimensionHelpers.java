package com.google.googlejavaformat.java;

import static java.util.Objects.requireNonNull;
import com.google.common.collect.ImmutableList;
import com.sun.source.tree.AnnotatedTypeTree;
import com.sun.source.tree.AnnotationTree;
import com.sun.source.tree.ArrayTypeTree;
import com.sun.source.tree.Tree;
import com.sun.tools.javac.tree.JCTree;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Deque;
import java.util.List;
import org.jspecify.annotations.Nullable;

final class DimensionHelpers {
    record TypeWithDims(@Nullable Tree node, ImmutableList<List<AnnotationTree>> dims) {
        TypeWithDims {
            requireNonNull(dims, "dims");
        }
    }
    enum SortedDims {
        YES, NO
    }
    static TypeWithDims extractDims(Tree node, SortedDims sorted) {
        Deque<List<AnnotationTree>> builder = new ArrayDeque<>();
        node = extractDims(builder, node);
        Iterable<List<AnnotationTree>> dims = sorted == SortedDims.YES ? reorderBySourcePosition(builder) : builder;
        return new TypeWithDims(node, ImmutableList.copyOf(dims));
    }
    private static Iterable<List<AnnotationTree>> reorderBySourcePosition(Deque<List<AnnotationTree>> dims) {
        int lastAnnotation = -1;
        int lastPos = -1;
        int idx = 0;
        for (List<AnnotationTree> dim : dims) {
            if (!dim.isEmpty()) {
                int pos = ((JCTree) dim.get(0)).getStartPosition();
                if (pos < lastPos) {
                    List<List<AnnotationTree>> list = new ArrayList<>(dims);
                    Collections.rotate(list, -(lastAnnotation + 1));
                    return list;
                }
                lastPos = pos;
                lastAnnotation = idx;
            }
            idx++;
        }
        return dims;
    }
    private static Tree extractDims(Deque<List<AnnotationTree>> dims, Tree node) {
        return switch (node.getKind()) {
            case ARRAY_TYPE -> extractDims(dims, ((ArrayTypeTree) node).getType());
            case ANNOTATED_TYPE -> {
                AnnotatedTypeTree annotatedTypeTree = (AnnotatedTypeTree) node;
                if (annotatedTypeTree.getUnderlyingType().getKind() != Tree.Kind.ARRAY_TYPE) {
                    yield node;
                }
                node = extractDims(dims, annotatedTypeTree.getUnderlyingType());
                dims.addFirst(ImmutableList.copyOf(annotatedTypeTree.getAnnotations()));
                yield node;
            }
            default -> node;
        };
    }
    private DimensionHelpers() {}
}
