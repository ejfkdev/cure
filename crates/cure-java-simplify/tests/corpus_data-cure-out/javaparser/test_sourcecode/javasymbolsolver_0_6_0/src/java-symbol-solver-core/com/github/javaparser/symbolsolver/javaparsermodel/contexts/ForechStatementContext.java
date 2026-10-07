package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.ForeachStmt;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.List;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class ForechStatementContext extends AbstractJavaParserContext<ForeachStmt> {
    public ForechStatementContext(ForeachStmt wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        if (wrappedNode.getVariable().getVariables().size() != 1) {
            throw new IllegalStateException();
        }
        VariableDeclarator variableDeclarator = wrappedNode.getVariable().getVariables().get(0);
        return variableDeclarator.getName().getId().equals(name) ? SymbolReference.solved(JavaParserSymbolDeclaration.localVar(variableDeclarator, typeSolver)) : getParentNode(wrappedNode) instanceof BlockStmt ? StatementContext.solveInBlock(name, typeSolver, wrappedNode) : getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
}
