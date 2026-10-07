package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.ast.Node;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.LambdaArgumentTypePlaceholder;
import com.github.javaparser.symbolsolver.javaparsermodel.contexts.ContextHelper;
import com.github.javaparser.symbolsolver.logic.AbstractClassDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.comparators.MethodComparator;
import com.github.javaparser.symbolsolver.resolution.MethodResolutionLogic;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.*;
import java.util.function.Predicate;
import java.util.stream.Collectors;

public class ReflectionClassDeclaration extends AbstractClassDeclaration {
    private Class<?> clazz;
    private TypeSolver typeSolver;
    private ReflectionClassAdapter reflectionClassAdapter;
    public ReflectionClassDeclaration(Class<?> clazz, TypeSolver typeSolver) {
        if (clazz == null) {
            throw new IllegalArgumentException("Class should not be null");
        }
        if (clazz.isInterface()) {
            throw new IllegalArgumentException("Class should not be an interface");
        }
        if (clazz.isPrimitive()) {
            throw new IllegalArgumentException("Class should not represent a primitive class");
        }
        if (clazz.isArray()) {
            throw new IllegalArgumentException("Class should not be an array");
        }
        if (clazz.isEnum()) {
            throw new IllegalArgumentException("Class should not be an enum");
        }
        this.clazz = clazz;
        this.typeSolver = typeSolver;
        this.reflectionClassAdapter = new ReflectionClassAdapter(clazz, typeSolver, this);
    }
    @Override
    public Set<MethodDeclaration> getDeclaredMethods() {
        return reflectionClassAdapter.getDeclaredMethods();
    }
    @Override
    public List<ReferenceType> getAncestors() {
        return reflectionClassAdapter.getAncestors();
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        ReflectionClassDeclaration that = (ReflectionClassDeclaration) o;
        return clazz.getCanonicalName().equals(that.clazz.getCanonicalName());
    }
    @Override
    public int hashCode() {
        return clazz.hashCode();
    }
    @Override
    public String getPackageName() {
        return clazz.getPackage() != null ? clazz.getPackage().getName() : null;
    }
    @Override
    public String getClassName() {
        String canonicalName = clazz.getCanonicalName();
        return canonicalName != null && getPackageName() != null ? canonicalName.substring(getPackageName().length() + 1, canonicalName.length()) : null;
    }
    @Override
    public String getQualifiedName() {
        return clazz.getCanonicalName();
    }
    @Deprecated
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly) {
        List<MethodDeclaration> methods = new ArrayList<>();
        Predicate<Method> staticFilter = (m) -> !staticOnly || staticOnly && Modifier.isStatic(m.getModifiers());
        for (Method method : Arrays.stream(clazz.getDeclaredMethods()).filter((m) -> m.getName().equals(name)).filter(staticFilter).sorted(new MethodComparator()).collect(Collectors.toList())) {
            if (method.isBridge() || method.isSynthetic()) 
                continue;
            MethodDeclaration methodDeclaration = new ReflectionMethodDeclaration(method, typeSolver);
            methods.add(methodDeclaration);
        }
        if (getSuperClass() != null) {
            ClassDeclaration superClass = (ClassDeclaration) getSuperClass().getTypeDeclaration();
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(superClass, name, argumentsTypes, staticOnly, typeSolver);
            if (ref.isSolved()) {
                methods.add(ref.getCorrespondingDeclaration());
            }
        }
        for (ReferenceType interfaceDeclaration : getInterfaces()) {
            SymbolReference<MethodDeclaration> ref = MethodResolutionLogic.solveMethodInType(interfaceDeclaration.getTypeDeclaration(), name, argumentsTypes, staticOnly, typeSolver);
            if (ref.isSolved()) {
                methods.add(ref.getCorrespondingDeclaration());
            }
        }
        return MethodResolutionLogic.findMostApplicable(methods, name, argumentsTypes, typeSolver);
    }
    @Override
    public String toString() {
        return "ReflectionClassDeclaration{clazz=" + getId() + '}';
    }
    public Type getUsage(Node node) {
        return new ReferenceTypeImpl(this, typeSolver);
    }
    public Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameterValues) {
        List<MethodUsage> methods = new ArrayList<>();
        for (Method method : Arrays.stream(clazz.getDeclaredMethods()).filter((m) -> m.getName().equals(name)).sorted(new MethodComparator()).collect(Collectors.toList())) {
            if (method.isBridge() || method.isSynthetic()) 
                continue;
            MethodUsage methodUsage = new MethodUsage(new ReflectionMethodDeclaration(method, typeSolver));
            for (int i = 0; i < getTypeParameters().size() && i < typeParameterValues.size(); i++) {
                TypeParameterDeclaration tpToReplace = getTypeParameters().get(i);
                Type newValue = typeParameterValues.get(i);
                methodUsage = methodUsage.replaceTypeParameter(tpToReplace, newValue);
            }
            methods.add(methodUsage);
        }
        if (getSuperClass() != null) {
            ClassDeclaration superClass = (ClassDeclaration) getSuperClass().getTypeDeclaration();
            Optional<MethodUsage> ref = ContextHelper.solveMethodAsUsage(superClass, name, argumentsTypes, typeSolver, invokationContext, typeParameterValues);
            if (ref.isPresent()) {
                methods.add(ref.get());
            }
        }
        for (ReferenceType interfaceDeclaration : getInterfaces()) {
            Optional<MethodUsage> ref = ContextHelper.solveMethodAsUsage(interfaceDeclaration.getTypeDeclaration(), name, argumentsTypes, typeSolver, invokationContext, typeParameterValues);
            if (ref.isPresent()) {
                methods.add(ref.get());
            }
        }
        return MethodResolutionLogic.findMostApplicableUsage(methods, name, argumentsTypes, typeSolver);
    }
    @Override
    public boolean canBeAssignedTo(ReferenceTypeDeclaration other) {
        if (other instanceof LambdaArgumentTypePlaceholder) {
            return isFunctionalInterface();
        }
        if (other.getQualifiedName().equals(getQualifiedName())) {
            return true;
        }
        if (this.clazz.getSuperclass() != null && new ReflectionClassDeclaration(clazz.getSuperclass(), typeSolver).canBeAssignedTo(other)) {
            return true;
        }
        for (Class<?> interfaze : clazz.getInterfaces()) {
            if (new ReflectionInterfaceDeclaration(interfaze, typeSolver).canBeAssignedTo(other)) {
                return true;
            }
        }
        return false;
    }
    @Override
    public boolean isAssignableBy(Type type) {
        return reflectionClassAdapter.isAssignableBy(type);
    }
    @Override
    public boolean isTypeParameter() {
        return false;
    }
    @Override
    public FieldDeclaration getField(String name) {
        return reflectionClassAdapter.getField(name);
    }
    @Override
    public List<FieldDeclaration> getAllFields() {
        return reflectionClassAdapter.getAllFields();
    }
    @Deprecated
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        for (Field field : clazz.getFields()) {
            if (field.getName().equals(name)) {
                return SymbolReference.solved(new ReflectionFieldDeclaration(field, typeSolver));
            }
        }
        return SymbolReference.unsolved(ValueDeclaration.class);
    }
    @Override
    public boolean hasDirectlyAnnotation(String canonicalName) {
        return reflectionClassAdapter.hasDirectlyAnnotation(canonicalName);
    }
    @Override
    public boolean hasField(String name) {
        return reflectionClassAdapter.hasField(name);
    }
    @Override
    public boolean isAssignableBy(ReferenceTypeDeclaration other) {
        return isAssignableBy(new ReferenceTypeImpl(other, typeSolver));
    }
    @Override
    public String getName() {
        return clazz.getSimpleName();
    }
    @Override
    public boolean isField() {
        return false;
    }
    @Override
    public boolean isParameter() {
        return false;
    }
    @Override
    public boolean isType() {
        return true;
    }
    @Override
    public boolean isClass() {
        return !clazz.isInterface();
    }
    @Override
    public ReferenceTypeImpl getSuperClass() {
        return reflectionClassAdapter.getSuperClass();
    }
    @Override
    public List<ReferenceType> getInterfaces() {
        return reflectionClassAdapter.getInterfaces();
    }
    @Override
    public boolean isInterface() {
        return clazz.isInterface();
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return reflectionClassAdapter.getTypeParameters();
    }
    @Override
    public AccessLevel accessLevel() {
        return ReflectionFactory.modifiersToAccessLevel(this.clazz.getModifiers());
    }
    @Override
    public List<ConstructorDeclaration> getConstructors() {
        return reflectionClassAdapter.getConstructors();
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return reflectionClassAdapter.containerType();
    }
    @Override
    public Set<ReferenceTypeDeclaration> internalTypes() {
        return Arrays.stream(this.clazz.getDeclaredClasses()).map((ic) -> ReflectionFactory.typeDeclarationFor(ic, typeSolver)).collect(Collectors.toSet());
    }
    @Override
    protected ReferenceType object() {
        return new ReferenceTypeImpl(typeSolver.solveType(Object.class.getCanonicalName()), typeSolver);
    }
}
