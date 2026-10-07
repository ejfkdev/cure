package com.github.javaparser.symbolsolver.core.resolution;

import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.List;
import java.util.Optional;

public interface Context {
    Context getParent();
    default Optional<Type> solveGenericType(String name, TypeSolver typeSolver) {
        return Optional.empty();
    }
    default SymbolReference<TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        Context parent = getParent();
        return parent == null ? SymbolReference.unsolved(ReferenceTypeDeclaration.class) : parent.solveType(name, typeSolver);
    }
    SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver);
    default Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        SymbolReference<? extends ValueDeclaration> ref = solveSymbol(name, typeSolver);
        if (ref.isSolved()) {
            Value value = Value.from(ref.getCorrespondingDeclaration());
            return Optional.of(value);
        } else {
            return Optional.empty();
        }
    }
    default SymbolReference<ConstructorDeclaration> solveConstructor(List<Type> argumentsTypes, TypeSolver typeSolver) {
        throw new IllegalArgumentException("Constructor resolution is available only on Class Context");
    }
    SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver);
    default Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> argumentsTypes, TypeSolver typeSolver) {
        SymbolReference<MethodDeclaration> methodSolved = solveMethod(name, argumentsTypes, false, typeSolver);
        if (methodSolved.isSolved()) {
            MethodDeclaration methodDeclaration = methodSolved.getCorrespondingDeclaration();
            MethodUsage methodUsage = ContextHelper.resolveTypeVariables(this, methodDeclaration, argumentsTypes);
            return Optional.of(methodUsage);
        } else {
            return Optional.empty();
        }
    }
}
