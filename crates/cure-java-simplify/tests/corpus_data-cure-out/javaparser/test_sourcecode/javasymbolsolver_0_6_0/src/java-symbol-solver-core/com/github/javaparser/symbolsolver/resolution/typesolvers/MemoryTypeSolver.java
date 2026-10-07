package com.github.javaparser.symbolsolver.resolution.typesolvers;

import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.HashMap;
import java.util.Map;

public class MemoryTypeSolver implements TypeSolver {
    private TypeSolver parent;
    private Map<String, ReferenceTypeDeclaration> declarationMap = new HashMap<>();
    @Override
    public String toString() {
        return "MemoryTypeSolver{parent=" + parent + ", declarationMap=" + declarationMap + '}';
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (!(o instanceof MemoryTypeSolver)) 
            return false;
        MemoryTypeSolver that = (MemoryTypeSolver) o;
        return (parent != null ? !parent.equals(that.parent) : that.parent != null) ? false : !(declarationMap != null ? !declarationMap.equals(that.declarationMap) : that.declarationMap != null);
    }
    @Override
    public int hashCode() {
        int result = parent != null ? parent.hashCode() : 0;
        result = 31 * result + (declarationMap != null ? declarationMap.hashCode() : 0);
        return result;
    }
    @Override
    public TypeSolver getParent() {
        return parent;
    }
    @Override
    public void setParent(TypeSolver parent) {
        this.parent = parent;
    }
    public void addDeclaration(String name, ReferenceTypeDeclaration typeDeclaration) {
        this.declarationMap.put(name, typeDeclaration);
    }
    @Override
    public SymbolReference<ReferenceTypeDeclaration> tryToSolveType(String name) {
        return declarationMap.containsKey(name) ? SymbolReference.solved(declarationMap.get(name)) : SymbolReference.unsolved(ReferenceTypeDeclaration.class);
    }
}
