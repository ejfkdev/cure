package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.body.ConstructorDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;

public class ConstructorContext extends AbstractMethodLikeDeclarationContext<ConstructorDeclaration> {
    public ConstructorContext(ConstructorDeclaration wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
}
