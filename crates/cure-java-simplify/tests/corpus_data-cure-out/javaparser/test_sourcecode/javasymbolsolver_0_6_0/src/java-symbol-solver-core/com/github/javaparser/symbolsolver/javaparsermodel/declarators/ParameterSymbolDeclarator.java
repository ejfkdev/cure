package com.github.javaparser.symbolsolver.javaparsermodel.declarators;

import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.LinkedList;
import java.util.List;

public class ParameterSymbolDeclarator extends AbstractSymbolDeclarator<Parameter> {
    public ParameterSymbolDeclarator(Parameter wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public List<ValueDeclaration> getSymbolDeclarations() {
        List<ValueDeclaration> symbols = new LinkedList<>();
        symbols.add(JavaParserSymbolDeclaration.parameter(wrappedNode, typeSolver));
        return symbols;
    }
}
