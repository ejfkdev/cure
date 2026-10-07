package com.github.javaparser.symbolsolver.javaparsermodel;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.*;
import com.github.javaparser.ast.expr.*;
import com.github.javaparser.ast.stmt.*;
import com.github.javaparser.ast.type.TypeParameter;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.contexts.*;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserAnnotationDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserClassDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserInterfaceDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserTypeParameter;
import com.github.javaparser.symbolsolver.javaparsermodel.declarators.FieldSymbolDeclarator;
import com.github.javaparser.symbolsolver.javaparsermodel.declarators.NoSymbolDeclarator;
import com.github.javaparser.symbolsolver.javaparsermodel.declarators.ParameterSymbolDeclarator;
import com.github.javaparser.symbolsolver.javaparsermodel.declarators.VariableSymbolDeclarator;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class JavaParserFactory {
    public static Context getContext(Node node, TypeSolver typeSolver) {
        if (node == null) {
            return null;
        } else if (node instanceof CompilationUnit) {
            return new CompilationUnitContext((CompilationUnit) node, typeSolver);
        } else if (node instanceof ForeachStmt) {
            return new ForechStatementContext((ForeachStmt) node, typeSolver);
        } else if (node instanceof ForStmt) {
            return new ForStatementContext((ForStmt) node, typeSolver);
        } else if (node instanceof LambdaExpr) {
            return new LambdaExprContext((LambdaExpr) node, typeSolver);
        } else if (node instanceof MethodDeclaration) {
            return new MethodContext((MethodDeclaration) node, typeSolver);
        } else if (node instanceof ConstructorDeclaration) {
            return new ConstructorContext((ConstructorDeclaration) node, typeSolver);
        } else if (node instanceof ClassOrInterfaceDeclaration) {
            return new ClassOrInterfaceDeclarationContext((ClassOrInterfaceDeclaration) node, typeSolver);
        } else if (node instanceof MethodCallExpr) {
            return new MethodCallExprContext((MethodCallExpr) node, typeSolver);
        } else if (node instanceof EnumDeclaration) {
            return new EnumDeclarationContext((EnumDeclaration) node, typeSolver);
        } else if (node instanceof FieldAccessExpr) {
            return new FieldAccessContext((FieldAccessExpr) node, typeSolver);
        } else if (node instanceof SwitchEntryStmt) {
            return new SwitchEntryContext((SwitchEntryStmt) node, typeSolver);
        } else if (node instanceof TryStmt) {
            return new TryWithResourceContext((TryStmt) node, typeSolver);
        } else if (node instanceof Statement) {
            return new StatementContext<Statement>((Statement) node, typeSolver);
        } else if (node instanceof CatchClause) {
            return new CatchClauseContext((CatchClause) node, typeSolver);
        } else if (node instanceof ObjectCreationExpr && ((ObjectCreationExpr) node).getAnonymousClassBody().isPresent()) {
            return new AnonymousClassDeclarationContext((ObjectCreationExpr) node, typeSolver);
        } else {
            if (node instanceof NameExpr) {
                if (node.getParentNode().isPresent() && node.getParentNode().get() instanceof FieldAccessExpr && node.getParentNode().get().getParentNode().isPresent()) {
                    return getContext(node.getParentNode().get().getParentNode().get(), typeSolver);
                }
            }
            Node parentNode = getParentNode(node);
            return parentNode instanceof ObjectCreationExpr && node == ((ObjectCreationExpr) parentNode).getType() ? getContext(getParentNode(parentNode), typeSolver) : getContext(parentNode, typeSolver);
        }
    }
    public static SymbolDeclarator getSymbolDeclarator(Node node, TypeSolver typeSolver) {
        if (node instanceof FieldDeclaration) {
            return new FieldSymbolDeclarator((FieldDeclaration) node, typeSolver);
        } else if (node instanceof Parameter) {
            return new ParameterSymbolDeclarator((Parameter) node, typeSolver);
        } else if (node instanceof ExpressionStmt) {
            ExpressionStmt expressionStmt = (ExpressionStmt) node;
            return expressionStmt.getExpression() instanceof VariableDeclarationExpr ? new VariableSymbolDeclarator((VariableDeclarationExpr) expressionStmt.getExpression(), typeSolver) : new NoSymbolDeclarator<ExpressionStmt>(expressionStmt, typeSolver);
        } else 
            return node instanceof IfStmt ? new NoSymbolDeclarator<IfStmt>((IfStmt) node, typeSolver) : node instanceof ForeachStmt ? new VariableSymbolDeclarator((VariableDeclarationExpr) ((ForeachStmt) node).getVariable(), typeSolver) : new NoSymbolDeclarator<Node>(node, typeSolver);
    }
    public static ReferenceTypeDeclaration toTypeDeclaration(Node node, TypeSolver typeSolver) {
        if (node instanceof ClassOrInterfaceDeclaration) {
            return ((ClassOrInterfaceDeclaration) node).isInterface() ? new JavaParserInterfaceDeclaration((ClassOrInterfaceDeclaration) node, typeSolver) : new JavaParserClassDeclaration((ClassOrInterfaceDeclaration) node, typeSolver);
        } else if (node instanceof TypeParameter) {
            return new JavaParserTypeParameter((TypeParameter) node, typeSolver);
        } else if (node instanceof EnumDeclaration) {
            return new JavaParserEnumDeclaration((EnumDeclaration) node, typeSolver);
        } else if (node instanceof AnnotationDeclaration) {
            return new JavaParserAnnotationDeclaration((AnnotationDeclaration) node, typeSolver);
        } else {
            throw new IllegalArgumentException(node.getClass().getCanonicalName());
        }
    }
}
