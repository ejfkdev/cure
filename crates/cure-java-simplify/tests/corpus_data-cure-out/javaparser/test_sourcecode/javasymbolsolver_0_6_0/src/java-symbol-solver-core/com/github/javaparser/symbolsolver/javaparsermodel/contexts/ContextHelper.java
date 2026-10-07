package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserClassDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserInterfaceDeclaration;
import com.github.javaparser.symbolsolver.javassistmodel.JavassistClassDeclaration;
import com.github.javaparser.symbolsolver.javassistmodel.JavassistEnumDeclaration;
import com.github.javaparser.symbolsolver.javassistmodel.JavassistInterfaceDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionClassDeclaration;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionEnumDeclaration;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionInterfaceDeclaration;
import java.util.List;
import java.util.Optional;

public class ContextHelper {
    private ContextHelper() {}
    public static Optional<MethodUsage> solveMethodAsUsage(TypeDeclaration typeDeclaration, String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameters) {
        if (typeDeclaration instanceof JavassistClassDeclaration) {
            return ((JavassistClassDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof JavassistInterfaceDeclaration) {
            return ((JavassistInterfaceDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof JavassistEnumDeclaration) {
            return ((JavassistEnumDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof ReflectionClassDeclaration) {
            return ((ReflectionClassDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof ReflectionInterfaceDeclaration) {
            return ((ReflectionInterfaceDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof ReflectionEnumDeclaration) {
            return ((ReflectionEnumDeclaration) typeDeclaration).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, typeParameters);
        } else if (typeDeclaration instanceof JavaParserClassDeclaration) {
            return ((JavaParserClassDeclaration) typeDeclaration).getContext().solveMethodAsUsage(name, argumentsTypes, typeSolver);
        } else if (typeDeclaration instanceof JavaParserInterfaceDeclaration) {
            return ((JavaParserInterfaceDeclaration) typeDeclaration).getContext().solveMethodAsUsage(name, argumentsTypes, typeSolver);
        } else if (typeDeclaration instanceof JavaParserEnumDeclaration) {
            return ((JavaParserEnumDeclaration) typeDeclaration).getContext().solveMethodAsUsage(name, argumentsTypes, typeSolver);
        }
        throw new UnsupportedOperationException(typeDeclaration.toString());
    }
}
