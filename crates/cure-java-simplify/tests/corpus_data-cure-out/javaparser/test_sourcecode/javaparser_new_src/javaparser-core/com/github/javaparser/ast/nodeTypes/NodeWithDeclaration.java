package com.github.javaparser.ast.nodeTypes;

public interface NodeWithDeclaration {
    String getDeclarationAsString();
    String getDeclarationAsString(boolean includingModifiers, boolean includingThrows);
    String getDeclarationAsString(boolean includingModifiers, boolean includingThrows, boolean includingParameterName);
}
