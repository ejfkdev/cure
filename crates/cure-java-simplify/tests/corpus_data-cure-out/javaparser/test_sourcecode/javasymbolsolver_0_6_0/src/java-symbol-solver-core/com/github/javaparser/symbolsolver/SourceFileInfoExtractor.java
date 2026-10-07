package com.github.javaparser.symbolsolver;

import com.github.javaparser.JavaParser;
import com.github.javaparser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.PackageDeclaration;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.ImportDeclaration;
import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.io.File;
import java.io.IOException;
import java.io.PrintStream;
import java.util.LinkedList;
import java.util.List;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class SourceFileInfoExtractor {
    private TypeSolver typeSolver;
    private int ok = 0;
    private int ko = 0;
    private int unsupported = 0;
    private boolean printFileName = true;
    private PrintStream out = System.out;
    private PrintStream err = System.err;
    public void setVerbose(boolean verbose) {
        this.verbose = verbose;
    }
    private boolean verbose = false;
    public void setPrintFileName(boolean printFileName) {
        this.printFileName = printFileName;
    }
    public void clear() {
        ok = 0;
        ko = 0;
        unsupported = 0;
    }
    public void setOut(PrintStream out) {
        this.out = out;
    }
    public void setErr(PrintStream err) {
        this.err = err;
    }
    public int getOk() {
        return ok;
    }
    public int getUnsupported() {
        return unsupported;
    }
    public int getKo() {
        return ko;
    }
    private void solveTypeDecl(ClassOrInterfaceDeclaration node) {
        TypeDeclaration typeDeclaration = JavaParserFacade.get(typeSolver).getTypeDeclaration(node);
        if (typeDeclaration.isClass()) {
            out.println("\n[ Class " + typeDeclaration.getQualifiedName() + " ]");
            for (ReferenceType sc : typeDeclaration.asClass().getAllSuperClasses()) {
                out.println("  superclass: " + sc.getQualifiedName());
            }
            for (ReferenceType sc : typeDeclaration.asClass().getAllInterfaces()) {
                out.println("  interface: " + sc.getQualifiedName());
            }
        }
    }
    private void solve(Node node) {
        if (node instanceof ClassOrInterfaceDeclaration) {
            solveTypeDecl((ClassOrInterfaceDeclaration) node);
        } else if (node instanceof Expression) {
            if (!(getParentNode(node) instanceof ImportDeclaration || getParentNode(node) instanceof Expression || getParentNode(node) instanceof MethodDeclaration || getParentNode(node) instanceof PackageDeclaration)) 
                if (getParentNode(node) instanceof Statement || getParentNode(node) instanceof VariableDeclarator) {
                    try {
                        Type ref = JavaParserFacade.get(typeSolver).getType(node);
                        out.println("  Line " + node.getRange().get().begin.line + ") " + node + " ==> " + ref.describe());
                        ok++;
                    } catch (UnsupportedOperationException upe) {
                        unsupported++;
                        err.println(upe.getMessage());
                        throw upe;
                    } catch (RuntimeException re) {
                        ko++;
                        err.println(re.getMessage());
                        throw re;
                    }
                }
        }
    }
    private void solveMethodCalls(Node node) {
        if (node instanceof MethodCallExpr) {
            out.println("  Line " + node.getBegin().get().line + ") " + node + " ==> " + toString((MethodCallExpr) node));
        }
        for (Node child : node.getChildNodes()) {
            solveMethodCalls(child);
        }
    }
    private String toString(MethodCallExpr node) {
        try {
            return toString(JavaParserFacade.get(typeSolver).solve(node));
        } catch (Exception e) {
            if (verbose) {
                System.err.println("Error resolving call at L" + node.getBegin().get().line + ": " + node);
                e.printStackTrace();
            }
            return "ERROR";
        }
    }
    private String toString(SymbolReference<com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration> methodDeclarationSymbolReference) {
        return methodDeclarationSymbolReference.isSolved() ? methodDeclarationSymbolReference.getCorrespondingDeclaration().getQualifiedSignature() : "UNSOLVED";
    }
    private List<Node> collectAllNodes(Node node) {
        List<Node> nodes = new LinkedList<>();
        collectAllNodes(node, nodes);
        nodes.sort((n1, n2) -> n1.getBegin().get().compareTo(n2.getBegin().get()));
        return nodes;
    }
    private void collectAllNodes(Node node, List<Node> nodes) {
        nodes.add(node);
        node.getChildNodes().forEach((c) -> collectAllNodes(c, nodes));
    }
    public void solve(File file) throws IOException, ParseException {
        if (file.isDirectory()) {
            for (File f : file.listFiles()) {
                solve(f);
            }
        } else {
            if (file.getName().endsWith(".java")) {
                if (printFileName) {
                    out.println("- parsing " + file.getAbsolutePath());
                }
                collectAllNodes(JavaParser.parse(file)).forEach((n) -> solve(n));
            }
        }
    }
    public void solveMethodCalls(File file) throws IOException, ParseException {
        if (file.isDirectory()) {
            for (File f : file.listFiles()) {
                solveMethodCalls(f);
            }
        } else {
            if (file.getName().endsWith(".java")) {
                if (printFileName) {
                    out.println("- parsing " + file.getAbsolutePath());
                }
                solveMethodCalls(JavaParser.parse(file));
            }
        }
    }
    public void setTypeSolver(TypeSolver typeSolver) {
        this.typeSolver = typeSolver;
    }
}
