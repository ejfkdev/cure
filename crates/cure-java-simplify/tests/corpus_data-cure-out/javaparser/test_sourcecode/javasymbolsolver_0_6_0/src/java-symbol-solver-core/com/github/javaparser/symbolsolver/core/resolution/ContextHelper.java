package com.github.javaparser.symbolsolver.core.resolution;

import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserMethodDeclaration;
import com.github.javaparser.symbolsolver.javassistmodel.JavassistMethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionMethodDeclaration;
import java.util.List;

class ContextHelper {
    private ContextHelper() {}
    static MethodUsage resolveTypeVariables(Context context, MethodDeclaration methodDeclaration, List<Type> parameterTypes) {
        if (methodDeclaration instanceof JavaParserMethodDeclaration) {
            return ((JavaParserMethodDeclaration) methodDeclaration).resolveTypeVariables(context, parameterTypes);
        } else if (methodDeclaration instanceof JavassistMethodDeclaration) {
            return ((JavassistMethodDeclaration) methodDeclaration).resolveTypeVariables(context, parameterTypes);
        } else if (methodDeclaration instanceof JavaParserEnumDeclaration.ValuesMethod) {
            return ((JavaParserEnumDeclaration.ValuesMethod) methodDeclaration).resolveTypeVariables(context, parameterTypes);
        } else if (methodDeclaration instanceof ReflectionMethodDeclaration) {
            return ((ReflectionMethodDeclaration) methodDeclaration).resolveTypeVariables(context, parameterTypes);
        } else {
            throw new UnsupportedOperationException();
        }
    }
}
