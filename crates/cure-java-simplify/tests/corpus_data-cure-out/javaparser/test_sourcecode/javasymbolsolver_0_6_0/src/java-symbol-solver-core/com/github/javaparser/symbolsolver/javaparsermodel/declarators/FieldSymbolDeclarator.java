package com.github.javaparser.symbolsolver.javaparsermodel.declarators;

import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.LinkedList;
import java.util.List;

public class FieldSymbolDeclarator extends AbstractSymbolDeclarator<FieldDeclaration> {
    public FieldSymbolDeclarator(FieldDeclaration wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public List<ValueDeclaration> getSymbolDeclarations() {
        List<ValueDeclaration> symbols = new LinkedList<>();
        for (VariableDeclarator v : wrappedNode.getVariables()) {
            symbols.add(JavaParserSymbolDeclaration.field(v, typeSolver));
        }
        return symbols;
    }
}
