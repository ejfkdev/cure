package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.TryStmt;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserSymbolDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import java.util.List;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class TryWithResourceContext extends AbstractJavaParserContext<TryStmt> {
    public TryWithResourceContext(TryStmt wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        for (VariableDeclarationExpr expr : wrappedNode.getResources()) {
            for (VariableDeclarator v : expr.getVariables()) {
                if (v.getName().getIdentifier().equals(name)) {
                    JavaParserSymbolDeclaration decl = JavaParserSymbolDeclaration.localVar(v, typeSolver);
                    return Optional.of(Value.from(decl));
                }
            }
        }
        return getParentNode(wrappedNode) instanceof BlockStmt ? StatementContext.solveInBlockAsValue(name, typeSolver, wrappedNode) : getParent().solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        for (VariableDeclarationExpr expr : wrappedNode.getResources()) {
            for (VariableDeclarator v : expr.getVariables()) {
                if (v.getName().getIdentifier().equals(name)) {
                    return SymbolReference.solved(JavaParserSymbolDeclaration.localVar(v, typeSolver));
                }
            }
        }
        return getParentNode(wrappedNode) instanceof BlockStmt ? StatementContext.solveInBlock(name, typeSolver, wrappedNode) : getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
}
