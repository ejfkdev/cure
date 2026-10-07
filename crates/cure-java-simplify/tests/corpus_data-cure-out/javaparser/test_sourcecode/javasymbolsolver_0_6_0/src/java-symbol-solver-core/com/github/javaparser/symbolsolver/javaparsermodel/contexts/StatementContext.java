package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.expr.LambdaExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithStatements;
import com.github.javaparser.ast.stmt.IfStmt;
import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import java.util.List;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class StatementContext<N extends Statement> extends AbstractJavaParserContext<N> {
    public StatementContext(N wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    public static SymbolReference<? extends ValueDeclaration> solveInBlock(String name, TypeSolver typeSolver, Statement stmt) {
        if (!(getParentNode(stmt) instanceof NodeWithStatements)) {
            throw new IllegalArgumentException();
        }
        NodeWithStatements<?> blockStmt = (NodeWithStatements<?>) getParentNode(stmt);
        int position = -1;
        for (int i = 0; i < blockStmt.getStatements().size(); i++) {
            if (blockStmt.getStatements().get(i).equals(stmt)) {
                position = i;
            }
        }
        if (position == -1) {
            throw new RuntimeException();
        }
        for (int i = position - 1; i >= 0; i--) {
            SymbolReference<? extends ValueDeclaration> symbolReference = solveWith(JavaParserFactory.getSymbolDeclarator(blockStmt.getStatements().get(i), typeSolver), name);
            if (symbolReference.isSolved()) {
                return symbolReference;
            }
        }
        return JavaParserFactory.getContext(getParentNode(stmt), typeSolver).solveSymbol(name, typeSolver);
    }
    public static Optional<Value> solveInBlockAsValue(String name, TypeSolver typeSolver, Statement stmt) {
        if (!(getParentNode(stmt) instanceof NodeWithStatements)) {
            throw new IllegalArgumentException();
        }
        NodeWithStatements<?> blockStmt = (NodeWithStatements<?>) getParentNode(stmt);
        int position = -1;
        for (int i = 0; i < blockStmt.getStatements().size(); i++) {
            if (blockStmt.getStatements().get(i).equals(stmt)) {
                position = i;
            }
        }
        if (position == -1) {
            throw new RuntimeException();
        }
        for (int i = position - 1; i >= 0; i--) {
            SymbolReference<? extends ValueDeclaration> symbolReference = solveWith(JavaParserFactory.getSymbolDeclarator(blockStmt.getStatements().get(i), typeSolver), name);
            if (symbolReference.isSolved()) {
                return Optional.of(Value.from(symbolReference.getCorrespondingDeclaration()));
            }
        }
        return JavaParserFactory.getContext(getParentNode(stmt), typeSolver).solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        SymbolDeclarator symbolDeclarator = JavaParserFactory.getSymbolDeclarator(wrappedNode, typeSolver);
        Optional<Value> symbolReference = solveWithAsValue(symbolDeclarator, name, typeSolver);
        if (symbolReference.isPresent()) {
            return symbolReference;
        }
        if (getParentNode(wrappedNode) instanceof com.github.javaparser.ast.body.MethodDeclaration) {
            return getParent().solveSymbolAsValue(name, typeSolver);
        }
        if (getParentNode(wrappedNode) instanceof LambdaExpr) {
            return getParent().solveSymbolAsValue(name, typeSolver);
        }
        if (getParentNode(wrappedNode) instanceof IfStmt) {
            return getParent().solveSymbolAsValue(name, typeSolver);
        }
        if (!(getParentNode(wrappedNode) instanceof NodeWithStatements)) {
            return getParent().solveSymbolAsValue(name, typeSolver);
        }
        NodeWithStatements<?> nodeWithStmt = (NodeWithStatements<?>) getParentNode(wrappedNode);
        int position = -1;
        for (int i = 0; i < nodeWithStmt.getStatements().size(); i++) {
            if (nodeWithStmt.getStatements().get(i).equals(wrappedNode)) {
                position = i;
            }
        }
        if (position == -1) {
            throw new RuntimeException();
        }
        for (int i = position - 1; i >= 0; i--) {
            symbolDeclarator = JavaParserFactory.getSymbolDeclarator(nodeWithStmt.getStatements().get(i), typeSolver);
            symbolReference = solveWithAsValue(symbolDeclarator, name, typeSolver);
            if (symbolReference.isPresent()) {
                return symbolReference;
            }
        }
        return getParent().solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        SymbolDeclarator symbolDeclarator = JavaParserFactory.getSymbolDeclarator(wrappedNode, typeSolver);
        SymbolReference<? extends ValueDeclaration> symbolReference = solveWith(symbolDeclarator, name);
        if (symbolReference.isSolved()) {
            return symbolReference;
        }
        if (getParentNode(wrappedNode) instanceof com.github.javaparser.ast.body.MethodDeclaration) {
            return getParent().solveSymbol(name, typeSolver);
        }
        if (getParentNode(wrappedNode) instanceof com.github.javaparser.ast.body.ConstructorDeclaration) {
            return getParent().solveSymbol(name, typeSolver);
        }
        if (getParentNode(wrappedNode) instanceof LambdaExpr) {
            return getParent().solveSymbol(name, typeSolver);
        }
        if (!(getParentNode(wrappedNode) instanceof NodeWithStatements)) {
            return getParent().solveSymbol(name, typeSolver);
        }
        NodeWithStatements<?> nodeWithStmt = (NodeWithStatements<?>) getParentNode(wrappedNode);
        int position = -1;
        for (int i = 0; i < nodeWithStmt.getStatements().size(); i++) {
            if (nodeWithStmt.getStatements().get(i).equals(wrappedNode)) {
                position = i;
            }
        }
        if (position == -1) {
            throw new RuntimeException();
        }
        for (int i = position - 1; i >= 0; i--) {
            symbolDeclarator = JavaParserFactory.getSymbolDeclarator(nodeWithStmt.getStatements().get(i), typeSolver);
            symbolReference = solveWith(symbolDeclarator, name);
            if (symbolReference.isSolved()) {
                return symbolReference;
            }
        }
        return getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
    @Override
    public SymbolReference<TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        return getParent().solveType(name, typeSolver);
    }
}
