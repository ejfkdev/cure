package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.body.EnumConstantDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.symbolsolver.javaparser.Navigator;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import com.github.javaparser.symbolsolver.model.declarations.FieldDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class JavaParserFieldDeclaration implements FieldDeclaration {
    private VariableDeclarator variableDeclarator;
    private com.github.javaparser.ast.body.FieldDeclaration wrappedNode;
    private EnumConstantDeclaration enumConstantDeclaration;
    private TypeSolver typeSolver;
    public JavaParserFieldDeclaration(VariableDeclarator variableDeclarator, TypeSolver typeSolver) {
        if (typeSolver == null) {
            throw new IllegalArgumentException("typeSolver should not be null");
        }
        this.variableDeclarator = variableDeclarator;
        this.typeSolver = typeSolver;
        if (!(getParentNode(variableDeclarator) instanceof com.github.javaparser.ast.body.FieldDeclaration)) {
            throw new IllegalStateException(getParentNode(variableDeclarator).getClass().getCanonicalName());
        }
        this.wrappedNode = (com.github.javaparser.ast.body.FieldDeclaration) getParentNode(variableDeclarator);
    }
    public JavaParserFieldDeclaration(EnumConstantDeclaration enumConstantDeclaration, TypeSolver typeSolver) {
        if (typeSolver == null) {
            throw new IllegalArgumentException("typeSolver should not be null");
        }
        this.enumConstantDeclaration = enumConstantDeclaration;
        this.typeSolver = typeSolver;
    }
    @Override
    public Type getType() {
        return enumConstantDeclaration != null ? new ReferenceTypeImpl(new JavaParserEnumDeclaration((com.github.javaparser.ast.body.EnumDeclaration) getParentNode(enumConstantDeclaration), typeSolver), typeSolver) : JavaParserFacade.get(typeSolver).convert(variableDeclarator.getType(), wrappedNode);
    }
    @Override
    public String getName() {
        return enumConstantDeclaration != null ? enumConstantDeclaration.getName().getId() : variableDeclarator.getName().getId();
    }
    @Override
    public boolean isStatic() {
        return wrappedNode.getModifiers().contains(Modifier.STATIC);
    }
    @Override
    public boolean isField() {
        return true;
    }
    public com.github.javaparser.ast.body.FieldDeclaration getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public String toString() {
        return "JPField{" + getName() + "}";
    }
    @Override
    public AccessLevel accessLevel() {
        return Helper.toAccessLevel(wrappedNode.getModifiers());
    }
    @Override
    public TypeDeclaration declaringType() {
        Optional<com.github.javaparser.ast.body.TypeDeclaration> typeDeclaration = Navigator.findAncestor(wrappedNode, com.github.javaparser.ast.body.TypeDeclaration.class);
        if (typeDeclaration.isPresent()) {
            return JavaParserFacade.get(typeSolver).getTypeDeclaration(typeDeclaration.get());
        } else {
            throw new IllegalStateException();
        }
    }
}
