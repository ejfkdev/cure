package com.github.javaparser.symbolsolver.reflectionmodel;

import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import com.github.javaparser.symbolsolver.model.declarations.FieldDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

public class ReflectionFieldDeclaration implements FieldDeclaration {
    private Field field;
    private TypeSolver typeSolver;
    private Type type;
    public ReflectionFieldDeclaration(Field field, TypeSolver typeSolver) {
        this.field = field;
        this.typeSolver = typeSolver;
        this.type = calcType();
    }
    private ReflectionFieldDeclaration(Field field, TypeSolver typeSolver, Type type) {
        this.field = field;
        this.typeSolver = typeSolver;
        this.type = type;
    }
    @Override
    public Type getType() {
        return type;
    }
    private Type calcType() {
        return ReflectionFactory.typeUsageFor(field.getGenericType(), typeSolver);
    }
    @Override
    public String getName() {
        return field.getName();
    }
    @Override
    public boolean isStatic() {
        return Modifier.isStatic(field.getModifiers());
    }
    @Override
    public boolean isField() {
        return true;
    }
    @Override
    public TypeDeclaration declaringType() {
        return ReflectionFactory.typeDeclarationFor(field.getDeclaringClass(), typeSolver);
    }
    public FieldDeclaration replaceType(Type fieldType) {
        return new ReflectionFieldDeclaration(field, typeSolver, fieldType);
    }
    @Override
    public boolean isParameter() {
        return false;
    }
    @Override
    public boolean isType() {
        return false;
    }
    @Override
    public AccessLevel accessLevel() {
        return ReflectionFactory.modifiersToAccessLevel(field.getModifiers());
    }
}
