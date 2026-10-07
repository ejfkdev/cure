package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.stmt.CatchClause;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import java.util.List;
import java.util.Optional;

public class CatchClauseContext extends AbstractJavaParserContext<CatchClause> {
    public CatchClauseContext(CatchClause wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    public final SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        SymbolDeclarator sb = JavaParserFactory.getSymbolDeclarator(wrappedNode.getParameter(), typeSolver);
        SymbolReference<? extends ValueDeclaration> symbolReference = AbstractJavaParserContext.solveWith(sb, name);
        return symbolReference.isSolved() ? symbolReference : getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public final Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        Optional<Value> symbolReference = solveWithAsValue(JavaParserFactory.getSymbolDeclarator(wrappedNode.getParameter(), typeSolver), name, typeSolver);
        return symbolReference.isPresent() ? symbolReference : getParent().solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public final SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
}
