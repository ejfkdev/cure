package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.model.declarations.ParameterDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ArrayType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;

public class JavaParserParameterDeclaration implements ParameterDeclaration {
    private Parameter wrappedNode;
    private TypeSolver typeSolver;
    public JavaParserParameterDeclaration(Parameter wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    @Override
    public boolean isField() {
        return false;
    }
    @Override
    public boolean isParameter() {
        return true;
    }
    @Override
    public boolean isVariadic() {
        return wrappedNode.isVarArgs();
    }
    @Override
    public boolean isType() {
        throw new UnsupportedOperationException();
    }
    @Override
    public Type getType() {
        Type res = JavaParserFacade.get(typeSolver).convert(wrappedNode.getType(), wrappedNode);
        if (isVariadic()) {
            res = new ArrayType(res);
        }
        return res;
    }
    @Override
    public ParameterDeclaration asParameter() {
        return this;
    }
    public Parameter getWrappedNode() {
        return wrappedNode;
    }
}
