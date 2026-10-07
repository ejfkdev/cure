package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.body.EnumConstantDeclaration;
import com.github.javaparser.ast.body.EnumDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumConstantDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.List;

public class EnumDeclarationContext extends AbstractJavaParserContext<EnumDeclaration> {
    private JavaParserTypeDeclarationAdapter javaParserTypeDeclarationAdapter;
    public EnumDeclarationContext(EnumDeclaration wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
        this.javaParserTypeDeclarationAdapter = new JavaParserTypeDeclarationAdapter(wrappedNode, typeSolver, getDeclaration(), this);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        if (typeSolver == null) 
            throw new IllegalArgumentException();
        for (EnumConstantDeclaration constant : wrappedNode.getEntries()) {
            if (constant.getName().getId().equals(name)) {
                return SymbolReference.solved(new JavaParserEnumConstantDeclaration(constant, typeSolver));
            }
        }
        return this.getDeclaration().hasField(name) ? SymbolReference.solved(this.getDeclaration().getField(name)) : getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        return javaParserTypeDeclarationAdapter.solveType(name, typeSolver);
    }
    @Override
    public SymbolReference<com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return javaParserTypeDeclarationAdapter.solveMethod(name, argumentsTypes, staticOnly, typeSolver);
    }
    private ReferenceTypeDeclaration getDeclaration() {
        return new JavaParserEnumDeclaration(this.wrappedNode, typeSolver);
    }
}
