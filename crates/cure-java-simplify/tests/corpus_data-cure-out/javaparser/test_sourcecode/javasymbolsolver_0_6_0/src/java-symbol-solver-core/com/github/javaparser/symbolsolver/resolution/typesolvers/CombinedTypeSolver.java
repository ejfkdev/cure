package com.github.javaparser.symbolsolver.resolution.typesolvers;

import com.github.javaparser.symbolsolver.javaparsermodel.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.ArrayList;
import java.util.List;

public class CombinedTypeSolver implements TypeSolver {
    private TypeSolver parent;
    private List<TypeSolver> elements = new ArrayList<>();
    public CombinedTypeSolver(TypeSolver... elements) {
        for (TypeSolver el : elements) {
            add(el);
        }
    }
    @Override
    public TypeSolver getParent() {
        return parent;
    }
    @Override
    public void setParent(TypeSolver parent) {
        this.parent = parent;
    }
    public void add(TypeSolver typeSolver) {
        this.elements.add(typeSolver);
        typeSolver.setParent(this);
    }
    @Override
    public SymbolReference<ReferenceTypeDeclaration> tryToSolveType(String name) {
        for (TypeSolver ts : elements) {
            SymbolReference<ReferenceTypeDeclaration> res = ts.tryToSolveType(name);
            if (res.isSolved()) {
                return res;
            }
        }
        return SymbolReference.unsolved(ReferenceTypeDeclaration.class);
    }
    @Override
    public ReferenceTypeDeclaration solveType(String name) throws UnsolvedSymbolException {
        SymbolReference<ReferenceTypeDeclaration> res = tryToSolveType(name);
        if (res.isSolved()) {
            return res.getCorrespondingDeclaration();
        } else {
            throw new UnsolvedSymbolException(name);
        }
    }
}
