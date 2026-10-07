package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.body.EnumDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class JavaParserEnumConstantDeclaration implements ValueDeclaration {
    private TypeSolver typeSolver;
    private com.github.javaparser.ast.body.EnumConstantDeclaration wrappedNode;
    public JavaParserEnumConstantDeclaration(com.github.javaparser.ast.body.EnumConstantDeclaration wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public Type getType() {
        return new ReferenceTypeImpl(new JavaParserEnumDeclaration((EnumDeclaration) getParentNode(wrappedNode), typeSolver), typeSolver);
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    public com.github.javaparser.ast.body.EnumConstantDeclaration getWrappedNode() {
        return wrappedNode;
    }
}
