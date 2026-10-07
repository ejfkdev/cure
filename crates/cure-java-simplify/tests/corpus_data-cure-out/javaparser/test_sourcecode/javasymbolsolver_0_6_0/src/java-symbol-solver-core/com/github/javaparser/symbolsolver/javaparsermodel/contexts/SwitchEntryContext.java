package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.ast.stmt.SwitchEntryStmt;
import com.github.javaparser.ast.stmt.SwitchStmt;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolDeclarator;
import java.util.List;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class SwitchEntryContext extends AbstractJavaParserContext<SwitchEntryStmt> {
    public SwitchEntryContext(SwitchEntryStmt wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        SwitchStmt switchStmt = (SwitchStmt) getParentNode(wrappedNode);
        Type type = JavaParserFacade.get(typeSolver).getType(switchStmt.getSelector());
        if (type.isReferenceType() && type.asReferenceType().getTypeDeclaration().isEnum()) {
            if (type instanceof ReferenceTypeImpl) {
                ReferenceTypeImpl typeUsageOfTypeDeclaration = (ReferenceTypeImpl) type;
                if (typeUsageOfTypeDeclaration.getTypeDeclaration().hasField(name)) {
                    return SymbolReference.solved(typeUsageOfTypeDeclaration.getTypeDeclaration().getField(name));
                }
            } else {
                throw new UnsupportedOperationException();
            }
        }
        for (SwitchEntryStmt seStmt : switchStmt.getEntries()) {
            if (!seStmt.equals(wrappedNode)) {
                for (Statement stmt : seStmt.getStatements()) {
                    SymbolReference<? extends ValueDeclaration> symbolReference = solveWith(JavaParserFactory.getSymbolDeclarator(stmt, typeSolver), name);
                    if (symbolReference.isSolved()) {
                        return symbolReference;
                    }
                }
            }
        }
        return getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        return getParent().solveMethod(name, argumentsTypes, false, typeSolver);
    }
}
