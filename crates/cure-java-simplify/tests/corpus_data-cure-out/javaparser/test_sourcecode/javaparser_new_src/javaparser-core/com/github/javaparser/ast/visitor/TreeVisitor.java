package com.github.javaparser.ast.visitor;

import com.github.javaparser.ast.Node;

public abstract class TreeVisitor {
    public void visitDepthFirst(Node node) {
        process(node);
        for (Node child : node.getChildrenNodes()) {
            visitDepthFirst(child);
        }
    }
    public abstract void process(Node node);
}
