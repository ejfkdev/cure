package com.github.javaparser.symbolsolver.model.resolution;

public class UnsolvedSymbolException extends RuntimeException {
    private String context;
    private String name;
    private TypeSolver typeSolver;
    public UnsolvedSymbolException(String name, TypeSolver typeSolver) {
        super("Unsolved symbol : " + name + " using TypeSolver " + typeSolver);
        this.typeSolver = typeSolver;
        this.name = name;
    }
    public UnsolvedSymbolException(String name, String context) {
        super("Unsolved symbol in " + context + " : " + name);
        this.context = context;
        this.name = name;
    }
    public UnsolvedSymbolException(String name) {
        super("Unsolved symbol : " + name);
        this.context = "unknown";
        this.name = name;
    }
    @Override
    public String toString() {
        return "UnsolvedSymbolException{context='" + context + '\'' + ", name='" + name + '\'' + ", typeSolver=" + typeSolver + '}';
    }
}
