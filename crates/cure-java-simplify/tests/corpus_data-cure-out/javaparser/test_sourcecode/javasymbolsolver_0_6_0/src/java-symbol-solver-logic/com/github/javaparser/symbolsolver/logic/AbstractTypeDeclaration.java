package com.github.javaparser.symbolsolver.logic;

import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

public abstract class AbstractTypeDeclaration implements ReferenceTypeDeclaration {
    @Override
    public final Set<MethodUsage> getAllMethods() {
        Set<MethodUsage> methods = new HashSet<>();
        Set<String> methodsSignatures = new HashSet<>();
        for (MethodDeclaration methodDeclaration : getDeclaredMethods()) {
            methods.add(new MethodUsage(methodDeclaration));
            methodsSignatures.add(methodDeclaration.getSignature());
        }
        for (ReferenceType ancestor : getAllAncestors()) {
            for (MethodUsage mu : ancestor.getDeclaredMethods()) {
                String signature = mu.getDeclaration().getSignature();
                if (!methodsSignatures.contains(signature)) {
                    methodsSignatures.add(signature);
                    methods.add(mu);
                }
            }
        }
        return methods;
    }
    @Override
    public final boolean isFunctionalInterface() {
        return FunctionalInterfaceLogic.getFunctionalMethod(this).isPresent();
    }
}
