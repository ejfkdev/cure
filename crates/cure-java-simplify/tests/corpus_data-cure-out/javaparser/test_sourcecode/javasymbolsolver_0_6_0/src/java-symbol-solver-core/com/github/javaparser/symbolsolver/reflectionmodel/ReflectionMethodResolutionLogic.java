package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.TypeVariable;
import com.github.javaparser.symbolsolver.resolution.MethodResolutionLogic;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Predicate;
import java.util.stream.Collectors;

class ReflectionMethodResolutionLogic {
    static SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> parameterTypes, boolean staticOnly, TypeSolver typeSolver, ReferenceTypeDeclaration scopeType, Class clazz) {
        List<MethodDeclaration> methods = new ArrayList<>();
        Predicate<Method> staticOnlyCheck = (m) -> !staticOnly || staticOnly && Modifier.isStatic(m.getModifiers());
        for (Method method : clazz.getMethods()) {
            if (method.isBridge() || method.isSynthetic() || !method.getName().equals(name) || !staticOnlyCheck.test(method)) 
                continue;
            MethodDeclaration methodDeclaration = new ReflectionMethodDeclaration(method, typeSolver);
            methods.add(methodDeclaration);
        }
        for (ReferenceType ancestor : scopeType.getAncestors()) {
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(ancestor.getTypeDeclaration(), name, parameterTypes, staticOnly, typeSolver);
            if (ref.isSolved()) {
                methods.add(ref.getCorrespondingDeclaration());
            }
        }
        if (scopeType.getAncestors().isEmpty()) {
            ReferenceTypeImpl objectClass = new ReferenceTypeImpl(new ReflectionClassDeclaration(Object.class, typeSolver), typeSolver);
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(objectClass.getTypeDeclaration(), name, parameterTypes, staticOnly, typeSolver);
            if (ref.isSolved()) {
                methods.add(ref.getCorrespondingDeclaration());
            }
        }
        return MethodResolutionLogic.findMostApplicable(methods, name, parameterTypes, typeSolver);
    }
    static Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameterValues, ReferenceTypeDeclaration scopeType, Class clazz) {
        if (typeParameterValues.size() != scopeType.getTypeParameters().size()) {
            if (!scopeType.getTypeParameters().isEmpty()) {
                typeParameterValues = new ArrayList<>();
                for (int i = 0; i < scopeType.getTypeParameters().size(); i++) {
                    typeParameterValues.add(new ReferenceTypeImpl(new ReflectionClassDeclaration(Object.class, typeSolver), typeSolver));
                }
            }
        }
        List<MethodUsage> methods = new ArrayList<>();
        for (Method method : clazz.getMethods()) {
            if (method.getName().equals(name) && !method.isBridge() && !method.isSynthetic()) {
                MethodUsage methodUsage = replaceParams(typeParameterValues, scopeType, new ReflectionMethodDeclaration(method, typeSolver));
                methods.add(methodUsage);
            }
        }
        for (ReferenceType ancestor : scopeType.getAncestors()) {
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(ancestor.getTypeDeclaration(), name, argumentsTypes, typeSolver);
            if (ref.isSolved()) {
                MethodDeclaration correspondingDeclaration = ref.getCorrespondingDeclaration();
                MethodUsage methodUsage = replaceParams(typeParameterValues, ancestor.getTypeDeclaration(), correspondingDeclaration);
                methods.add(methodUsage);
            }
        }
        if (scopeType.getAncestors().isEmpty()) {
            ReferenceTypeImpl objectClass = new ReferenceTypeImpl(new ReflectionClassDeclaration(Object.class, typeSolver), typeSolver);
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(objectClass.getTypeDeclaration(), name, argumentsTypes, typeSolver);
            if (ref.isSolved()) {
                MethodUsage usage = replaceParams(typeParameterValues, objectClass.getTypeDeclaration(), ref.getCorrespondingDeclaration());
                methods.add(usage);
            }
        }
        argumentsTypes = argumentsTypes.stream().map((pt) -> {
            int i = 0;
            for (TypeParameterDeclaration tp : scopeType.getTypeParameters()) {
                pt = pt.replaceTypeVariables(tp, typeParameterValues.get(i));
                i++;
            }
            return pt;
        }).collect(Collectors.toList());
        return MethodResolutionLogic.findMostApplicableUsage(methods, name, argumentsTypes, typeSolver);
    }
    private static MethodUsage replaceParams(List<Type> typeParameterValues, ReferenceTypeDeclaration typeParametrizable, MethodDeclaration methodDeclaration) {
        MethodUsage methodUsage = new MethodUsage(methodDeclaration);
        int i = 0;
        if (typeParameterValues.size() == typeParametrizable.getTypeParameters().size()) {
            for (TypeParameterDeclaration tp : typeParametrizable.getTypeParameters()) {
                methodUsage = methodUsage.replaceTypeParameter(tp, typeParameterValues.get(i));
                i++;
            }
        }
        for (TypeParameterDeclaration methodTypeParameter : methodDeclaration.getTypeParameters()) {
            methodUsage = methodUsage.replaceTypeParameter(methodTypeParameter, new TypeVariable(methodTypeParameter));
        }
        return methodUsage;
    }
}
