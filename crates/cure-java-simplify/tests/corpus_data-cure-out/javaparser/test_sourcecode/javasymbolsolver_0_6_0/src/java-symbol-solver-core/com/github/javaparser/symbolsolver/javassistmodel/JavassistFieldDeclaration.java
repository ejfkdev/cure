package com.github.javaparser.symbolsolver.javassistmodel;

import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import com.github.javaparser.symbolsolver.model.declarations.FieldDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import javassist.CtField;
import javassist.NotFoundException;
import java.lang.reflect.Modifier;

public class JavassistFieldDeclaration implements FieldDeclaration {
    private CtField ctField;
    private TypeSolver typeSolver;
    public JavassistFieldDeclaration(CtField ctField, TypeSolver typeSolver) {
        this.ctField = ctField;
        this.typeSolver = typeSolver;
    }
    @Override
    public Type getType() {
        try {
            return JavassistFactory.typeUsageFor(ctField.getType(), typeSolver);
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
    }
    @Override
    public boolean isStatic() {
        return Modifier.isStatic(ctField.getModifiers());
    }
    @Override
    public String getName() {
        return ctField.getName();
    }
    @Override
    public boolean isField() {
        return true;
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
        return JavassistFactory.modifiersToAccessLevel(ctField.getModifiers());
    }
    @Override
    public TypeDeclaration declaringType() {
        return JavassistFactory.toTypeDeclaration(ctField.getDeclaringClass(), typeSolver);
    }
}
