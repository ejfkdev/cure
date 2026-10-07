package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.FieldAccessExpr;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public abstract class AbstractJavaParserContext<N extends Node> implements Context {
    protected N wrappedNode;
    protected TypeSolver typeSolver;
    public static final SymbolReference<ValueDeclaration> solveWith(SymbolDeclarator symbolDeclarator, String name) {
        for (ValueDeclaration decl : symbolDeclarator.getSymbolDeclarations()) {
            if (decl.getName().equals(name)) {
                return SymbolReference.solved(decl);
            }
        }
        return SymbolReference.unsolved(ValueDeclaration.class);
    }
    public AbstractJavaParserContext(N wrappedNode, TypeSolver typeSolver) {
        if (wrappedNode == null) {
            throw new NullPointerException();
        }
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        AbstractJavaParserContext<?> that = (AbstractJavaParserContext<?>) o;
        return !(wrappedNode != null ? !wrappedNode.equals(that.wrappedNode) : that.wrappedNode != null);
    }
    @Override
    public int hashCode() {
        return wrappedNode != null ? wrappedNode.hashCode() : 0;
    }
    @Override
    public Optional<Type> solveGenericType(String name, TypeSolver typeSolver) {
        Context parent = getParent();
        return parent == null ? Optional.empty() : parent.solveGenericType(name, typeSolver);
    }
    @Override
    public final Context getParent() {
        if (getParentNode(wrappedNode) instanceof MethodCallExpr) {
            MethodCallExpr parentCall = (MethodCallExpr) getParentNode(wrappedNode);
            boolean found = false;
            if (parentCall.getArguments() != null) {
                for (Expression expression : parentCall.getArguments()) {
                    if (expression == wrappedNode) {
                        found = true;
                    }
                }
            }
            if (found) {
                Node notMethod = getParentNode(wrappedNode);
                while (notMethod instanceof MethodCallExpr) {
                    notMethod = getParentNode(notMethod);
                }
                return JavaParserFactory.getContext(notMethod, typeSolver);
            }
        }
        Node notMethod = getParentNode(wrappedNode);
        while (notMethod instanceof MethodCallExpr || notMethod instanceof FieldAccessExpr) {
            notMethod = getParentNode(notMethod);
        }
        return JavaParserFactory.getContext(notMethod, typeSolver);
    }
    protected Optional<Value> solveWithAsValue(SymbolDeclarator symbolDeclarator, String name, TypeSolver typeSolver) {
        return symbolDeclarator.getSymbolDeclarations().stream().filter((d) -> d.getName().equals(name)).map((d) -> Value.from(d)).findFirst();
    }
}
