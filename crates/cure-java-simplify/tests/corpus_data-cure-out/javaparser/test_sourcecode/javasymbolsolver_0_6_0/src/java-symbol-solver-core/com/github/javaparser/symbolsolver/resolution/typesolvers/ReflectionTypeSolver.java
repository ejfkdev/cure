package com.github.javaparser.symbolsolver.resolution.typesolvers;

import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionFactory;
import java.util.Optional;

public class ReflectionTypeSolver implements TypeSolver {
    private TypeSolver parent;
    public ReflectionTypeSolver(boolean jreOnly) {
        this.jreOnly = jreOnly;
    }
    public ReflectionTypeSolver() {
        this(true);
    }
    private boolean jreOnly;
    @Override
    public TypeSolver getParent() {
        return parent;
    }
    @Override
    public void setParent(TypeSolver parent) {
        this.parent = parent;
    }
    @Override
    public SymbolReference<ReferenceTypeDeclaration> tryToSolveType(String name) {
        if (!jreOnly || (name.startsWith("java.") || name.startsWith("javax."))) {
            try {
                ClassLoader classLoader = ReflectionTypeSolver.class.getClassLoader();
                if (classLoader == null) {
                    throw new RuntimeException("The ReflectionTypeSolver has been probably loaded through the bootstrap class loader. This usage is not supported by the JavaSymbolSolver");
                }
                Class<?> clazz = classLoader.loadClass(name);
                return SymbolReference.solved(ReflectionFactory.typeDeclarationFor(clazz, getRoot()));
            } catch (ClassNotFoundException e) {
                int lastDot = name.lastIndexOf('.');
                if (lastDot == -1) {
                    return SymbolReference.unsolved(ReferenceTypeDeclaration.class);
                } else {
                    String parentName = name.substring(0, lastDot);
                    String childName = name.substring(lastDot + 1);
                    SymbolReference<ReferenceTypeDeclaration> parent = tryToSolveType(parentName);
                    if (parent.isSolved()) {
                        Optional<ReferenceTypeDeclaration> innerClass = parent.getCorrespondingDeclaration().internalTypes().stream().filter((it) -> it.getName().equals(childName)).findFirst();
                        return innerClass.isPresent() ? SymbolReference.solved(innerClass.get()) : SymbolReference.unsolved(ReferenceTypeDeclaration.class);
                    } else {
                        return SymbolReference.unsolved(ReferenceTypeDeclaration.class);
                    }
                }
            }
        } else {
            return SymbolReference.unsolved(ReferenceTypeDeclaration.class);
        }
    }
}
