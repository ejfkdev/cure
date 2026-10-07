package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.TypeParameter;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.logic.AbstractTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.FieldDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.Set;

public class JavaParserTypeVariableDeclaration extends AbstractTypeDeclaration {
    private TypeParameter wrappedNode;
    private TypeSolver typeSolver;
    public JavaParserTypeVariableDeclaration(TypeParameter wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public boolean isAssignableBy(ReferenceTypeDeclaration other) {
        return isAssignableBy(new ReferenceTypeImpl(other, typeSolver));
    }
    @Override
    public String getPackageName() {
        return Helper.getPackageName(wrappedNode);
    }
    @Override
    public String getClassName() {
        return Helper.getClassName("", wrappedNode);
    }
    @Override
    public String getQualifiedName() {
        return getName();
    }
    public Context getContext() {
        throw new UnsupportedOperationException();
    }
    @Override
    public String toString() {
        return "JavaParserTypeVariableDeclaration{" + wrappedNode.getName() + '}';
    }
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> parameterTypes) {
        throw new UnsupportedOperationException();
    }
    public Type getUsage(Node node) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isAssignableBy(Type type) {
        if (type.isTypeVariable()) {
            throw new UnsupportedOperationException("Is this type variable declaration assignable by " + type.describe());
        } else {
            throw new UnsupportedOperationException("Is this type variable declaration assignable by " + type);
        }
    }
    @Override
    public boolean isTypeParameter() {
        return true;
    }
    @Override
    public FieldDeclaration getField(String name) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean hasField(String name) {
        return false;
    }
    @Override
    public List<FieldDeclaration> getAllFields() {
        return new ArrayList<>();
    }
    @Override
    public List<ReferenceType> getAncestors() {
        throw new UnsupportedOperationException();
    }
    @Override
    public Set<MethodDeclaration> getDeclaredMethods() {
        return Collections.emptySet();
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    @Override
    public boolean isField() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isParameter() {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isType() {
        return true;
    }
    @Override
    public boolean hasDirectlyAnnotation(String canonicalName) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean isClass() {
        return false;
    }
    @Override
    public boolean isInterface() {
        return false;
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return Collections.emptyList();
    }
    public TypeParameterDeclaration asTypeParameter() {
        return new JavaParserTypeParameter(this.wrappedNode, typeSolver);
    }
    public TypeParameter getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return asTypeParameter().containerType();
    }
}
