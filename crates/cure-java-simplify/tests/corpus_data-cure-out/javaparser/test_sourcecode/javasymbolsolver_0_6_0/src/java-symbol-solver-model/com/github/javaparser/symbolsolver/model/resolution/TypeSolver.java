package com.github.javaparser.symbolsolver.model.resolution;

import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;

public interface TypeSolver {
    default TypeSolver getRoot() {
        return getParent() == null ? this : getParent().getRoot();
    }
    TypeSolver getParent();
    void setParent(TypeSolver parent);
    SymbolReference<ReferenceTypeDeclaration> tryToSolveType(String name);
    default ReferenceTypeDeclaration solveType(String name) throws UnsolvedSymbolException {
        SymbolReference<ReferenceTypeDeclaration> ref = tryToSolveType(name);
        if (ref.isSolved()) {
            return ref.getCorrespondingDeclaration();
        } else {
            throw new UnsolvedSymbolException(name, this);
        }
    }
}
