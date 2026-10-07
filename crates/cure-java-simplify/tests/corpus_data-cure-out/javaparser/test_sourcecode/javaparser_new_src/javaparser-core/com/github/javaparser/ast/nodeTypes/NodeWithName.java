package com.github.javaparser.ast.nodeTypes;

public interface NodeWithName<T> {
    String getName();
    T setName(String name);
}
