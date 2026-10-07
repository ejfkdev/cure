package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.logic.AbstractTypeDeclaration;
import com.github.javaparser.symbolsolver.logic.ConfilictingGenericTypesException;
import com.github.javaparser.symbolsolver.logic.InferenceContext;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.*;

public class ReflectionEnumDeclaration extends AbstractTypeDeclaration implements EnumDeclaration {
    private Class<?> clazz;
    private TypeSolver typeSolver;
    private ReflectionClassAdapter reflectionClassAdapter;
    public ReflectionEnumDeclaration(Class<?> clazz, TypeSolver typeSolver) {
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
        if (!clazz.isEnum()) {
            throw new IllegalArgumentException("Class should be an enum");
        }
        this.clazz = clazz;
        this.typeSolver = typeSolver;
        this.reflectionClassAdapter = new ReflectionClassAdapter(clazz, typeSolver, this);
    }
    @Override
  public AccessLevel accessLevel() {
        return ReflectionFactory.modifiersToAccessLevel(this.clazz.getModifiers());
    }
    @Override
  public Optional<ReferenceTypeDeclaration> containerType() {
        return reflectionClassAdapter.containerType();
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
    @Override
  public List<ReferenceType> getAncestors() {
        return reflectionClassAdapter.getAncestors();
    }
    @Override
  public FieldDeclaration getField(String name) {
        return reflectionClassAdapter.getField(name);
    }
    @Override
  public boolean hasField(String name) {
        return reflectionClassAdapter.hasField(name);
    }
    @Override
  public List<FieldDeclaration> getAllFields() {
        return reflectionClassAdapter.getAllFields();
    }
    @Override
  public Set<MethodDeclaration> getDeclaredMethods() {
        return reflectionClassAdapter.getDeclaredMethods();
    }
    @Override
  public boolean isAssignableBy(Type type) {
        return reflectionClassAdapter.isAssignableBy(type);
    }
    @Override
  public boolean isAssignableBy(ReferenceTypeDeclaration other) {
        return isAssignableBy(new ReferenceTypeImpl(other, typeSolver));
    }
    @Override
  public boolean hasDirectlyAnnotation(String qualifiedName) {
        return reflectionClassAdapter.hasDirectlyAnnotation(qualifiedName);
    }
    @Override
  public String getName() {
        return clazz.getSimpleName();
    }
    @Override
  public List<TypeParameterDeclaration> getTypeParameters() {
        return reflectionClassAdapter.getTypeParameters();
    }
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> parameterTypes, boolean staticOnly) {
        return ReflectionMethodResolutionLogic.solveMethod(name, parameterTypes, staticOnly, typeSolver, this, clazz);
    }
    public Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> parameterTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameterValues) {
        Optional<MethodUsage> res = ReflectionMethodResolutionLogic.solveMethodAsUsage(name, parameterTypes, typeSolver, invokationContext, typeParameterValues, this, clazz);
        if (res.isPresent()) {
            InferenceContext inferenceContext = new InferenceContext(MyObjectProvider.INSTANCE);
            MethodUsage methodUsage = res.get();
            int i = 0;
            List<Type> parameters = new LinkedList<>();
            for (Type actualType : parameterTypes) {
                Type formalType = methodUsage.getParamType(i);
                parameters.add(inferenceContext.addPair(formalType, actualType));
                i++;
            }
            try {
                Type returnType = inferenceContext.addSingle(methodUsage.returnType());
                for (int j = 0; j < parameters.size(); j++) {
                    methodUsage = methodUsage.replaceParamType(j, inferenceContext.resolve(parameters.get(j)));
                }
                methodUsage = methodUsage.replaceReturnType(inferenceContext.resolve(returnType));
                return Optional.of(methodUsage);
            } catch (ConfilictingGenericTypesException e) {
                return Optional.empty();
            }
        } else {
            return res;
        }
    }
}
