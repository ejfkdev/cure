package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.logic.AbstractTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.MethodResolutionLogic;
import javassist.CtClass;
import javassist.CtMethod;
import javassist.NotFoundException;
import javassist.bytecode.AccessFlag;
import javassist.bytecode.SyntheticAttribute;
import java.lang.reflect.Modifier;
import java.util.*;
import java.util.function.Predicate;
import java.util.stream.Collectors;

public class JavassistEnumDeclaration extends AbstractTypeDeclaration implements EnumDeclaration {
    private CtClass ctClass;
    private TypeSolver typeSolver;
    private JavassistTypeDeclarationAdapter javassistTypeDeclarationAdapter;
    public JavassistEnumDeclaration(CtClass ctClass, TypeSolver typeSolver) {
        if (ctClass == null) {
            throw new IllegalArgumentException();
        }
        if (!ctClass.isEnum()) {
            throw new IllegalArgumentException("Trying to instantiate a JavassistEnumDeclaration with something which is not an enum: " + ctClass.toString());
        }
        this.ctClass = ctClass;
        this.typeSolver = typeSolver;
        this.javassistTypeDeclarationAdapter = new JavassistTypeDeclarationAdapter(ctClass, typeSolver);
    }
    @Override
    public AccessLevel accessLevel() {
        return JavassistFactory.modifiersToAccessLevel(ctClass.getModifiers());
    }
    @Override
    public String getPackageName() {
        return ctClass.getPackageName();
    }
    @Override
    public String getClassName() {
        String name = ctClass.getName().replace('$', '.');
        return getPackageName() != null ? name.substring(getPackageName().length() + 1, name.length()) : name;
    }
    @Override
    public String getQualifiedName() {
        return ctClass.getName().replace('$', '.');
    }
    @Override
    public List<ReferenceType> getAncestors() {
        List<ReferenceType> ancestors = new LinkedList<>();
        try {
            CtClass superClass = ctClass.getSuperclass();
            if (superClass != null) {
                Type superClassTypeUsage = JavassistFactory.typeUsageFor(superClass, typeSolver);
                if (superClassTypeUsage.isReferenceType()) {
                    ancestors.add(superClassTypeUsage.asReferenceType());
                }
            }
            for (CtClass interfaze : ctClass.getInterfaces()) {
                Type interfazeTypeUsage = JavassistFactory.typeUsageFor(interfaze, typeSolver);
                if (interfazeTypeUsage.isReferenceType()) {
                    ancestors.add(interfazeTypeUsage.asReferenceType());
                }
            }
        } catch (NotFoundException e) {
            throw new RuntimeException("Ancestor not found for " + ctClass.getName() + ".", e);
        }
        return ancestors;
    }
    @Override
    public FieldDeclaration getField(String name) {
        return javassistTypeDeclarationAdapter.getDeclaredFields().stream().filter((f) -> f.getName().equals(name)).findFirst().orElseThrow(() -> new RuntimeException("Field " + name + " does not exist in " + ctClass.getName() + "."));
    }
    @Override
    public boolean hasField(String name) {
        return javassistTypeDeclarationAdapter.getDeclaredFields().stream().anyMatch((f) -> f.getName().equals(name));
    }
    @Override
    public List<FieldDeclaration> getAllFields() {
        return javassistTypeDeclarationAdapter.getDeclaredFields();
    }
    @Override
    public Set<MethodDeclaration> getDeclaredMethods() {
        return javassistTypeDeclarationAdapter.getDeclaredMethods();
    }
    @Override
    public boolean isAssignableBy(Type type) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isAssignableBy(ReferenceTypeDeclaration other) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean hasDirectlyAnnotation(String canonicalName) {
        throw new UnsupportedOperationException();
    }
    @Override
    public String getName() {
        String[] nameElements = ctClass.getSimpleName().replace('$', '.').split("\\.");
        return nameElements[nameElements.length - 1];
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return javassistTypeDeclarationAdapter.getTypeParameters();
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return javassistTypeDeclarationAdapter.containerType();
    }
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly) {
        List<MethodDeclaration> candidates = new ArrayList<>();
        Predicate<CtMethod> staticOnlyCheck = (m) -> !staticOnly || staticOnly && Modifier.isStatic(m.getModifiers());
        for (CtMethod method : ctClass.getDeclaredMethods()) {
            boolean isSynthetic = method.getMethodInfo().getAttribute(SyntheticAttribute.tag) != null;
            boolean isNotBridge = (method.getMethodInfo().getAccessFlags() & AccessFlag.BRIDGE) == 0;
            if (method.getName().equals(name) && !isSynthetic && isNotBridge && staticOnlyCheck.test(method)) {
                candidates.add(new JavassistMethodDeclaration(method, typeSolver));
            }
        }
        try {
            CtClass superClass = ctClass.getSuperclass();
            if (superClass != null) {
                SymbolReference<MethodDeclaration> ref = new JavassistClassDeclaration(superClass, typeSolver).solveMethod(name, argumentsTypes, staticOnly);
                if (ref.isSolved()) {
                    candidates.add(ref.getCorrespondingDeclaration());
                }
            }
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
        return MethodResolutionLogic.findMostApplicable(candidates, name, argumentsTypes, typeSolver);
    }
    public Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameterValues) {
        return JavassistUtils.getMethodUsage(ctClass, name, argumentsTypes, typeSolver, invokationContext);
    }
    @Override
    public Set<ReferenceTypeDeclaration> internalTypes() {
        try {
            return Arrays.stream(ctClass.getDeclaredClasses()).map((itype) -> JavassistFactory.toTypeDeclaration(itype, typeSolver)).collect(Collectors.toSet());
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
    }
    @Override
    public ReferenceTypeDeclaration getInternalType(String name) {
        return this.internalTypes().stream().filter((f) -> f.getName().endsWith(name)).findFirst().orElseThrow(() -> new UnsolvedSymbolException("Internal type not found: " + name));
    }
    @Override
    public boolean hasInternalType(String name) {
        return this.internalTypes().stream().anyMatch((f) -> f.getName().endsWith(name));
    }
}
