package com.github.javaparser.symbolsolver.resolution;

import com.github.javaparser.symbolsolver.model.declarations.ConstructorDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodAmbiguityException;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ArrayType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

public class ConstructorResolutionLogic {
    private static List<Type> groupVariadicParamValues(List<Type> argumentsTypes, int startVariadic, Type variadicType) {
        List<Type> res = new ArrayList<>(argumentsTypes.subList(0, startVariadic));
        List<Type> variadicValues = argumentsTypes.subList(startVariadic, argumentsTypes.size());
        if (variadicValues.isEmpty()) {
            res.add(variadicType);
        } else {
            Type componentType = findCommonType(variadicValues);
            res.add(new ArrayType(componentType));
        }
        return res;
    }
    private static Type findCommonType(List<Type> variadicValues) {
        if (variadicValues.isEmpty()) {
            throw new IllegalArgumentException();
        }
        return variadicValues.get(0);
    }
    public static boolean isApplicable(ConstructorDeclaration constructor, List<Type> argumentsTypes, TypeSolver typeSolver) {
        return isApplicable(constructor, argumentsTypes, typeSolver, false);
    }
    private static boolean isApplicable(ConstructorDeclaration constructor, List<Type> argumentsTypes, TypeSolver typeSolver, boolean withWildcardTolerance) {
        if (constructor.hasVariadicParameter()) {
            int pos = constructor.getNumberOfParams() - 1;
            if (constructor.getNumberOfParams() == argumentsTypes.size()) {
                Type expectedType = constructor.getLastParam().getType();
                Type actualType = argumentsTypes.get(pos);
                if (!expectedType.isAssignableBy(actualType)) {
                    for (TypeParameterDeclaration tp : constructor.getTypeParameters()) {
                        expectedType = MethodResolutionLogic.replaceTypeParam(expectedType, tp, typeSolver);
                    }
                    if (!expectedType.isAssignableBy(actualType)) {
                        if (actualType.isArray() && expectedType.isAssignableBy(actualType.asArrayType().getComponentType())) {
                            argumentsTypes.set(pos, actualType.asArrayType().getComponentType());
                        } else {
                            argumentsTypes = groupVariadicParamValues(argumentsTypes, pos, constructor.getLastParam().getType());
                        }
                    }
                }
            } else {
                if (pos > argumentsTypes.size()) {
                    return false;
                }
                argumentsTypes = groupVariadicParamValues(argumentsTypes, pos, constructor.getLastParam().getType());
            }
        }
        if (constructor.getNumberOfParams() != argumentsTypes.size()) {
            return false;
        }
        Map<String, Type> matchedParameters = new HashMap<>();
        boolean needForWildCardTolerance = false;
        for (int i = 0; i < constructor.getNumberOfParams(); i++) {
            Type expectedType = constructor.getParam(i).getType();
            Type actualType = argumentsTypes.get(i);
            if (expectedType.isTypeVariable() && !expectedType.isWildcard() && expectedType.asTypeParameter().declaredOnMethod()) {
                matchedParameters.put(expectedType.asTypeParameter().getName(), actualType);
                continue;
            }
            boolean isAssignableWithoutSubstitution = expectedType.isAssignableBy(actualType) || constructor.getParam(i).isVariadic() && new ArrayType(expectedType).isAssignableBy(actualType);
            if (!isAssignableWithoutSubstitution && expectedType.isReferenceType() && actualType.isReferenceType()) {
                isAssignableWithoutSubstitution = MethodResolutionLogic.isAssignableMatchTypeParameters(expectedType.asReferenceType(), actualType.asReferenceType(), matchedParameters);
            }
            if (!isAssignableWithoutSubstitution) {
                List<TypeParameterDeclaration> typeParameters = constructor.getTypeParameters();
                typeParameters.addAll(constructor.declaringType().getTypeParameters());
                for (TypeParameterDeclaration tp : typeParameters) {
                    expectedType = MethodResolutionLogic.replaceTypeParam(expectedType, tp, typeSolver);
                }
                if (!expectedType.isAssignableBy(actualType)) {
                    if (actualType.isWildcard() && withWildcardTolerance && !expectedType.isPrimitive()) {
                        needForWildCardTolerance = true;
                        continue;
                    }
                    if (constructor.hasVariadicParameter() && i == constructor.getNumberOfParams() - 1) {
                        if (new ArrayType(expectedType).isAssignableBy(actualType)) {
                            continue;
                        }
                    }
                    return false;
                }
            }
        }
        return !withWildcardTolerance || needForWildCardTolerance;
    }
    public static SymbolReference<ConstructorDeclaration> findMostApplicable(List<ConstructorDeclaration> constructors, List<Type> argumentsTypes, TypeSolver typeSolver) {
        SymbolReference<ConstructorDeclaration> res = findMostApplicable(constructors, argumentsTypes, typeSolver, false);
        return res.isSolved() ? res : findMostApplicable(constructors, argumentsTypes, typeSolver, true);
    }
    public static SymbolReference<ConstructorDeclaration> findMostApplicable(List<ConstructorDeclaration> constructors, List<Type> argumentsTypes, TypeSolver typeSolver, boolean wildcardTolerance) {
        List<ConstructorDeclaration> applicableConstructors = constructors.stream().filter((m) -> isApplicable(m, argumentsTypes, typeSolver, wildcardTolerance)).collect(Collectors.toList());
        if (applicableConstructors.isEmpty()) {
            return SymbolReference.unsolved(ConstructorDeclaration.class);
        }
        if (applicableConstructors.size() == 1) {
            return SymbolReference.solved(applicableConstructors.get(0));
        } else {
            ConstructorDeclaration winningCandidate = applicableConstructors.get(0);
            ConstructorDeclaration other = null;
            boolean possibleAmbiguity = false;
            for (int i = 1; i < applicableConstructors.size(); i++) {
                other = applicableConstructors.get(i);
                if (isMoreSpecific(winningCandidate, other, typeSolver)) {
                    possibleAmbiguity = false;
                } else if (isMoreSpecific(other, winningCandidate, typeSolver)) {
                    possibleAmbiguity = false;
                    winningCandidate = other;
                } else {
                    if (winningCandidate.declaringType().getQualifiedName().equals(other.declaringType().getQualifiedName())) {
                        possibleAmbiguity = true;
                    }
                }
            }
            if (possibleAmbiguity) {
                if (!MethodResolutionLogic.isExactMatch(winningCandidate, argumentsTypes)) {
                    if (MethodResolutionLogic.isExactMatch(other, argumentsTypes)) {
                        winningCandidate = other;
                    } else {
                        throw new MethodAmbiguityException("Ambiguous constructor call: cannot find a most applicable constructor: " + winningCandidate + ", " + other);
                    }
                }
            }
            return SymbolReference.solved(winningCandidate);
        }
    }
    private static boolean isMoreSpecific(ConstructorDeclaration constructorA, ConstructorDeclaration constructorB, TypeSolver typeSolver) {
        boolean oneMoreSpecificFound = false;
        if (constructorA.getNumberOfParams() < constructorB.getNumberOfParams()) {
            return true;
        }
        if (constructorA.getNumberOfParams() > constructorB.getNumberOfParams()) {
            return false;
        }
        for (int i = 0; i < constructorA.getNumberOfParams(); i++) {
            Type tdA = constructorA.getParam(i).getType();
            Type tdB = constructorB.getParam(i).getType();
            if (tdB.isAssignableBy(tdA) && !tdA.isAssignableBy(tdB)) {
                oneMoreSpecificFound = true;
            }
            if (tdA.isAssignableBy(tdB) && !tdB.isAssignableBy(tdA)) {
                return false;
            }
            if (i == constructorA.getNumberOfParams() - 1 && tdA.arrayLevel() > tdB.arrayLevel()) {
                return true;
            }
        }
        return oneMoreSpecificFound;
    }
}
