package com.github.javaparser.symbolsolver.javaparsermodel.declarators;

import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import java.util.LinkedList;
import java.util.List;
import java.util.stream.Collectors;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class VariableSymbolDeclarator extends AbstractSymbolDeclarator<VariableDeclarationExpr> {
    public VariableSymbolDeclarator(VariableDeclarationExpr wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
        if (getParentNode(wrappedNode) instanceof FieldDeclaration) {
            throw new IllegalArgumentException();
        }
    }
    @Override
    public List<ValueDeclaration> getSymbolDeclarations() {
        return wrappedNode.getVariables().stream().map((v) -> JavaParserSymbolDeclaration.localVar(v, typeSolver)).collect(Collectors.toCollection(() -> new LinkedList<>()));
    }
}
