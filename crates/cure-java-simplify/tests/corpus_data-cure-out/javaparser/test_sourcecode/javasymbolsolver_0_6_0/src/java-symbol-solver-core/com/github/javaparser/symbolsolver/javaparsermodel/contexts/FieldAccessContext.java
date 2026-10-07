package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.FieldAccessExpr;
import com.github.javaparser.ast.expr.ThisExpr;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.PrimitiveType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import java.util.List;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

public class FieldAccessContext extends AbstractJavaParserContext<FieldAccessExpr> {
    private static final String ARRAY_LENGTH_FIELD_NAME = "length";
    public FieldAccessContext(FieldAccessExpr wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        if (wrappedNode.getField().toString().equals(name)) {
            if (wrappedNode.getScope() instanceof ThisExpr) {
                Type typeOfThis = JavaParserFacade.get(typeSolver).getTypeOfThisIn(wrappedNode);
                return new SymbolSolver(typeSolver).solveSymbolInType(typeOfThis.asReferenceType().getTypeDeclaration(), name);
            }
        }
        return JavaParserFactory.getContext(getParentNode(wrappedNode), typeSolver).solveSymbol(name, typeSolver);
    }
    @Override
    public SymbolReference<TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        return JavaParserFactory.getContext(getParentNode(wrappedNode), typeSolver).solveType(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> parameterTypes, boolean staticOnly, TypeSolver typeSolver) {
        return JavaParserFactory.getContext(getParentNode(wrappedNode), typeSolver).solveMethod(name, parameterTypes, false, typeSolver);
    }
    @Override
    public Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        Expression scope = wrappedNode.getScope();
        if (wrappedNode.getField().toString().equals(name)) {
            Type typeOfScope = JavaParserFacade.get(typeSolver).getType(scope);
            if (typeOfScope.isArray() && name.equals(ARRAY_LENGTH_FIELD_NAME)) {
                return Optional.of(new Value(PrimitiveType.INT, ARRAY_LENGTH_FIELD_NAME));
            }
            if (typeOfScope.isReferenceType()) {
                Optional<Type> typeUsage = typeOfScope.asReferenceType().getFieldType(name);
                return typeUsage.isPresent() ? Optional.of(new Value(typeUsage.get(), name)) : Optional.empty();
            } else {
                return Optional.empty();
            }
        } else {
            return getParent().solveSymbolAsValue(name, typeSolver);
        }
    }
}
