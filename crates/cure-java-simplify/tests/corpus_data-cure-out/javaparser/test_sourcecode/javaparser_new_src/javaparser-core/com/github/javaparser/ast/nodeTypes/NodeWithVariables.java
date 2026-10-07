package com.github.javaparser.ast.nodeTypes;

import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import java.util.List;

public interface NodeWithVariables<T> {
    List<VariableDeclarator> getVariables();
    T setVariables(List<VariableDeclarator> variables);
}
