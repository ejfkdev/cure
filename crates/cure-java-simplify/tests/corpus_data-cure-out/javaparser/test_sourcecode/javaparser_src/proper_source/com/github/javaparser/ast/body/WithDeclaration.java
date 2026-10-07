package com.github.javaparser.ast.body;

public interface WithDeclaration {
    String getDeclarationAsString();
    String getDeclarationAsString(boolean includingModifiers, boolean includingThrows);
    String getDeclarationAsString(boolean includingModifiers, boolean includingThrows, boolean includingParameterName);
}
