package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.*;
import javassist.CtClass;
import javassist.NotFoundException;
import java.lang.reflect.Modifier;

public class JavassistFactory {
    public static Type typeUsageFor(CtClass ctClazz, TypeSolver typeSolver) {
        try {
            return ctClazz.isArray() ? new ArrayType(typeUsageFor(ctClazz.getComponentType(), typeSolver)) : ctClazz.isPrimitive() ? ctClazz.getName().equals("void") ? VoidType.INSTANCE : PrimitiveType.byName(ctClazz.getName()) : ctClazz.isInterface() ? new ReferenceTypeImpl(new JavassistInterfaceDeclaration(ctClazz, typeSolver), typeSolver) : ctClazz.isEnum() ? new ReferenceTypeImpl(new JavassistEnumDeclaration(ctClazz, typeSolver), typeSolver) : new ReferenceTypeImpl(new JavassistClassDeclaration(ctClazz, typeSolver), typeSolver);
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
    }
    public static ReferenceTypeDeclaration toTypeDeclaration(CtClass ctClazz, TypeSolver typeSolver) {
        if (ctClazz.isInterface()) {
            return new JavassistInterfaceDeclaration(ctClazz, typeSolver);
        } else if (ctClazz.isEnum()) {
            return new JavassistEnumDeclaration(ctClazz, typeSolver);
        } else if (ctClazz.isAnnotation()) {
            throw new UnsupportedOperationException("CtClass of annotation not yet supported");
        } else if (ctClazz.isArray()) {
            throw new IllegalArgumentException("This method should not be called passing an array");
        } else {
            return new JavassistClassDeclaration(ctClazz, typeSolver);
        }
    }
    static AccessLevel modifiersToAccessLevel(final int modifiers) {
        return Modifier.isPublic(modifiers) ? AccessLevel.PUBLIC : Modifier.isProtected(modifiers) ? AccessLevel.PROTECTED : Modifier.isPrivate(modifiers) ? AccessLevel.PRIVATE : AccessLevel.PACKAGE_PROTECTED;
    }
}
