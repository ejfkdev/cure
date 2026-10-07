package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.*;
import java.lang.reflect.GenericArrayType;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.WildcardType;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class ReflectionFactory {
    public static ReferenceTypeDeclaration typeDeclarationFor(Class<?> clazz, TypeSolver typeSolver) {
        if (clazz.isArray()) {
            throw new IllegalArgumentException("No type declaration available for an Array");
        } else if (clazz.isPrimitive()) {
            throw new IllegalArgumentException();
        } else 
            return clazz.isInterface() ? new ReflectionInterfaceDeclaration(clazz, typeSolver) : clazz.isEnum() ? new ReflectionEnumDeclaration(clazz, typeSolver) : new ReflectionClassDeclaration(clazz, typeSolver);
    }
    public static Type typeUsageFor(java.lang.reflect.Type type, TypeSolver typeSolver) {
        if (type instanceof java.lang.reflect.TypeVariable) {
            java.lang.reflect.TypeVariable<?> tv = (java.lang.reflect.TypeVariable<?>) type;
            return new com.github.javaparser.symbolsolver.model.typesystem.TypeVariable(new ReflectionTypeParameter(tv, tv.getGenericDeclaration() instanceof java.lang.reflect.Type, typeSolver));
        } else if (type instanceof ParameterizedType) {
            ParameterizedType pt = (ParameterizedType) type;
            ReferenceType rawType = typeUsageFor(pt.getRawType(), typeSolver).asReferenceType();
            List<java.lang.reflect.Type> actualTypes = new ArrayList<>();
            actualTypes.addAll(Arrays.asList(pt.getActualTypeArguments()));
            rawType = rawType.transformTypeParameters((tp) -> typeUsageFor(actualTypes.remove(0), typeSolver)).asReferenceType();
            return rawType;
        } else if (type instanceof Class) {
            Class<?> c = (Class<?>) type;
            return c.isPrimitive() ? c.getName().equals(Void.TYPE.getName()) ? VoidType.INSTANCE : PrimitiveType.byName(c.getName()) : c.isArray() ? new ArrayType(typeUsageFor(c.getComponentType(), typeSolver)) : new ReferenceTypeImpl(typeDeclarationFor(c, typeSolver), typeSolver);
        } else if (type instanceof GenericArrayType) {
            return new ArrayType(typeUsageFor(((GenericArrayType) type).getGenericComponentType(), typeSolver));
        } else if (type instanceof WildcardType) {
            WildcardType wildcardType = (WildcardType) type;
            if (wildcardType.getLowerBounds().length > 0 && wildcardType.getUpperBounds().length > 0) {
                if (wildcardType.getUpperBounds().length == 1 && wildcardType.getUpperBounds()[0].getTypeName().equals("java.lang.Object")) {}
            }
            if (wildcardType.getLowerBounds().length > 0) {
                if (wildcardType.getLowerBounds().length > 1) {
                    throw new UnsupportedOperationException();
                }
                return Wildcard.superBound(typeUsageFor(wildcardType.getLowerBounds()[0], typeSolver));
            }
            if (wildcardType.getUpperBounds().length > 0) {
                if (wildcardType.getUpperBounds().length > 1) {
                    throw new UnsupportedOperationException();
                }
                return Wildcard.extendsBound(typeUsageFor(wildcardType.getUpperBounds()[0], typeSolver));
            }
            return Wildcard.UNBOUNDED;
        } else {
            throw new UnsupportedOperationException(type.getClass().getCanonicalName() + " " + type);
        }
    }
    static AccessLevel modifiersToAccessLevel(final int modifiers) {
        return Modifier.isPublic(modifiers) ? AccessLevel.PUBLIC : Modifier.isProtected(modifiers) ? AccessLevel.PROTECTED : Modifier.isPrivate(modifiers) ? AccessLevel.PRIVATE : AccessLevel.PACKAGE_PROTECTED;
    }
}
