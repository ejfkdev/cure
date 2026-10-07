package com.github.javaparser.symbolsolver.logic;

import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

public final class FunctionalInterfaceLogic {
    private FunctionalInterfaceLogic() {}
    public static Optional<MethodUsage> getFunctionalMethod(Type type) {
        return type.isReferenceType() && type.asReferenceType().getTypeDeclaration().isInterface() ? getFunctionalMethod(type.asReferenceType().getTypeDeclaration()) : Optional.empty();
    }
    public static Optional<MethodUsage> getFunctionalMethod(ReferenceTypeDeclaration typeDeclaration) {
        Set<MethodUsage> methods = typeDeclaration.getAllMethods().stream().filter((m) -> m.getDeclaration().isAbstract()).filter((m) -> !declaredOnObject(m)).collect(Collectors.toSet());
        return methods.size() == 1 ? Optional.of(methods.iterator().next()) : Optional.empty();
    }
    private static String getSignature(Method m) {
        return String.format("%s(%s)", m.getName(), String.join(", ", Arrays.stream(m.getParameters()).map((p) -> toSignature(p)).collect(Collectors.toList())));
    }
    private static String toSignature(Parameter p) {
        return p.getType().getCanonicalName();
    }
    private static List<String> OBJECT_METHODS_SIGNATURES = Arrays.stream(Object.class.getDeclaredMethods()).map((method) -> getSignature(method)).collect(Collectors.toList());
    private static boolean declaredOnObject(MethodUsage m) {
        return OBJECT_METHODS_SIGNATURES.contains(m.getDeclaration().getSignature());
    }
}
