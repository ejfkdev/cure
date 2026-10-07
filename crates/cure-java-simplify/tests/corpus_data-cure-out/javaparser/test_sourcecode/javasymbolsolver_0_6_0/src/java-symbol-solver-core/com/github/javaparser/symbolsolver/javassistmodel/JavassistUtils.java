package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParametrizable;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.model.typesystem.*;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import javassist.CtClass;
import javassist.CtMethod;
import javassist.NotFoundException;
import javassist.bytecode.BadBytecode;
import javassist.bytecode.SignatureAttribute;
import java.util.*;
import java.util.stream.Collectors;

class JavassistUtils {
    static Optional<MethodUsage> getMethodUsage(CtClass ctClass, String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext) {
        for (CtMethod method : ctClass.getDeclaredMethods()) {
            if (method.getName().equals(name)) {
                MethodUsage methodUsage = new MethodUsage(new JavassistMethodDeclaration(method, typeSolver));
                if (argumentsTypes.size() < methodUsage.getNoParams()) {
                    continue;
                }
                try {
                    if (method.getGenericSignature() != null) {
                        List<Type> parametersOfReturnType = parseTypeParameters(SignatureAttribute.toMethodSignature(method.getGenericSignature()).getReturnType().toString(), typeSolver, invokationContext);
                        Type newReturnType = methodUsage.returnType();
                        if (!(newReturnType instanceof VoidType)) {
                            newReturnType = newReturnType.asReferenceType().transformTypeParameters((tp) -> parametersOfReturnType.remove(0));
                        }
                        methodUsage = methodUsage.replaceReturnType(newReturnType);
                    }
                    return Optional.of(methodUsage);
                } catch (BadBytecode e) {
                    throw new RuntimeException(e);
                }
            }
        }
        try {
            CtClass superClass = ctClass.getSuperclass();
            if (superClass != null) {
                Optional<MethodUsage> ref = new JavassistClassDeclaration(superClass, typeSolver).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, null);
                if (ref.isPresent()) {
                    return ref;
                }
            }
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
        try {
            for (CtClass interfaze : ctClass.getInterfaces()) {
                Optional<MethodUsage> ref = new JavassistInterfaceDeclaration(interfaze, typeSolver).solveMethodAsUsage(name, argumentsTypes, typeSolver, invokationContext, null);
                if (ref.isPresent()) {
                    return ref;
                }
            }
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
        return Optional.empty();
    }
    private static List<Type> parseTypeParameters(String signature, TypeSolver typeSolver, Context invokationContext) {
        String originalSignature = signature;
        if (signature.contains("<")) {
            signature = signature.substring(signature.indexOf('<') + 1);
            if (!signature.endsWith(">")) {
                throw new IllegalArgumentException();
            }
            signature = signature.substring(0, signature.length() - 1);
            if (signature.contains(",")) {
                throw new UnsupportedOperationException();
            }
            if (signature.contains("<")) {
                throw new UnsupportedOperationException(originalSignature);
            }
            if (signature.contains(">")) {
                throw new UnsupportedOperationException();
            }
            List<Type> types = new ArrayList<>();
            types.add(new SymbolSolver(typeSolver).solveTypeUsage(signature, invokationContext));
            return types;
        } else {
            return Collections.emptyList();
        }
    }
    static Type signatureTypeToType(SignatureAttribute.Type signatureType, TypeSolver typeSolver, TypeParametrizable typeParametrizable) {
        if (signatureType instanceof SignatureAttribute.ClassType) {
            SignatureAttribute.ClassType classType = (SignatureAttribute.ClassType) signatureType;
            List<Type> typeParameters = classType.getTypeArguments() == null ? Collections.emptyList() : Arrays.stream(classType.getTypeArguments()).map((ta) -> typeArgumentToType(ta, typeSolver, typeParametrizable)).collect(Collectors.toList());
            String typeName = classType.getDeclaringClass() != null ? classType.getDeclaringClass().getName() + "." + classType.getName() : classType.getName();
            return new ReferenceTypeImpl(typeSolver.solveType(internalNameToCanonicalName(typeName)), typeParameters, typeSolver);
        } else if (signatureType instanceof SignatureAttribute.TypeVariable) {
            SignatureAttribute.TypeVariable typeVariableSignature = (SignatureAttribute.TypeVariable) signatureType;
            Optional<TypeParameterDeclaration> typeParameterDeclarationOpt = typeParametrizable.findTypeParameter(typeVariableSignature.getName());
            if (!typeParameterDeclarationOpt.isPresent()) {
                throw new UnsolvedSymbolException("Unable to solve TypeVariable " + typeVariableSignature);
            }
            return new TypeVariable(typeParameterDeclarationOpt.get());
        } else {
            throw new RuntimeException(signatureType.getClass().getCanonicalName());
        }
    }
    private static String internalNameToCanonicalName(String typeName) {
        return typeName.replaceAll("\\$", ".");
    }
    private static Type objectTypeArgumentToType(SignatureAttribute.ObjectType typeArgument, TypeSolver typeSolver, TypeParametrizable typeParametrizable) {
        String typeName = typeArgument.jvmTypeName();
        return getGenericParameterByName(typeName, typeParametrizable).orElseGet(() -> new ReferenceTypeImpl(typeSolver.solveType(internalNameToCanonicalName(typeName)), typeSolver));
    }
    private static Optional<Type> getGenericParameterByName(String typeName, TypeParametrizable typeParametrizable) {
        return typeParametrizable.findTypeParameter(typeName).map(TypeVariable::new);
    }
    private static Type typeArgumentToType(SignatureAttribute.TypeArgument typeArgument, TypeSolver typeSolver, TypeParametrizable typeParametrizable) {
        if (typeArgument.isWildcard()) {
            if (typeArgument.getType() == null) {
                return Wildcard.UNBOUNDED;
            } else if (typeArgument.getKind() == '+') {
                return Wildcard.extendsBound(objectTypeArgumentToType(typeArgument.getType(), typeSolver, typeParametrizable));
            } else if (typeArgument.getKind() == '-') {
                return Wildcard.superBound(objectTypeArgumentToType(typeArgument.getType(), typeSolver, typeParametrizable));
            } else {
                throw new UnsupportedOperationException();
            }
        } else {
            return objectTypeArgumentToType(typeArgument.getType(), typeSolver, typeParametrizable);
        }
    }
}
