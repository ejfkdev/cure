package com.github.javaparser.symbolsolver.javaparsermodel.declarators;

import com.github.javaparser.ast.Node;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;

public abstract class AbstractSymbolDeclarator<N extends Node> implements SymbolDeclarator {
    protected N wrappedNode;
    protected TypeSolver typeSolver;
    public AbstractSymbolDeclarator(N wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
}
