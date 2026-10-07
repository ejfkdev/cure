package com.github.javaparser.symbolsolver.resolution;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.javaparsermodel.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserClassDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserEnumDeclaration;
import com.github.javaparser.symbolsolver.javaparsermodel.declarations.JavaParserInterfaceDeclaration;
import com.github.javaparser.symbolsolver.javassistmodel.JavassistClassDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionClassDeclaration;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionInterfaceDeclaration;
import java.util.List;
import java.util.Optional;

public class SymbolSolver {
    private TypeSolver typeSolver;
    public SymbolSolver(TypeSolver typeSolver) {
        if (typeSolver == null) 
            throw new IllegalArgumentException();
        this.typeSolver = typeSolver;
    }
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, Context context) {
        return context.solveSymbol(name, typeSolver);
    }
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, Node node) {
        return solveSymbol(name, JavaParserFactory.getContext(node, typeSolver));
    }
    public Optional<Value> solveSymbolAsValue(String name, Context context) {
        return context.solveSymbolAsValue(name, typeSolver);
    }
    public Optional<Value> solveSymbolAsValue(String name, Node node) {
        return solveSymbolAsValue(name, JavaParserFactory.getContext(node, typeSolver));
    }
    public SymbolReference<? extends TypeDeclaration> solveType(String name, Context context) {
        return context.solveType(name, typeSolver);
    }
    public SymbolReference<? extends TypeDeclaration> solveType(String name, Node node) {
        return solveType(name, JavaParserFactory.getContext(node, typeSolver));
    }
    public MethodUsage solveMethod(String methodName, List<Type> argumentsTypes, Context context) {
        SymbolReference<MethodDeclaration> decl = context.solveMethod(methodName, argumentsTypes, false, typeSolver);
        if (!decl.isSolved()) {
            throw new UnsolvedSymbolException(context, methodName);
        }
        return new MethodUsage(decl.getCorrespondingDeclaration());
    }
    public MethodUsage solveMethod(String methodName, List<Type> argumentsTypes, Node node) {
        return solveMethod(methodName, argumentsTypes, JavaParserFactory.getContext(node, typeSolver));
    }
    public TypeDeclaration solveType(com.github.javaparser.ast.type.Type type) {
        if (type instanceof ClassOrInterfaceType) {
            String name = ((ClassOrInterfaceType) type).getName().getId();
            SymbolReference<TypeDeclaration> ref = JavaParserFactory.getContext(type, typeSolver).solveType(name, typeSolver);
            if (!ref.isSolved()) {
                throw new UnsolvedSymbolException(JavaParserFactory.getContext(type, typeSolver), name);
            }
            return ref.getCorrespondingDeclaration();
        } else {
            throw new UnsupportedOperationException(type.getClass().getCanonicalName());
        }
    }
    public Type solveTypeUsage(String name, Context context) {
        Optional<Type> genericType = context.solveGenericType(name, typeSolver);
        return genericType.isPresent() ? genericType.get() : new ReferenceTypeImpl(typeSolver.solveType(name), typeSolver);
    }
    public SymbolReference<? extends ValueDeclaration> solveSymbolInType(TypeDeclaration typeDeclaration, String name) {
        return typeDeclaration instanceof JavaParserClassDeclaration ? ((JavaParserClassDeclaration) typeDeclaration).getContext().solveSymbol(name, typeSolver) : typeDeclaration instanceof JavaParserInterfaceDeclaration ? ((JavaParserInterfaceDeclaration) typeDeclaration).getContext().solveSymbol(name, typeSolver) : typeDeclaration instanceof JavaParserEnumDeclaration ? ((JavaParserEnumDeclaration) typeDeclaration).getContext().solveSymbol(name, typeSolver) : typeDeclaration instanceof ReflectionClassDeclaration ? ((ReflectionClassDeclaration) typeDeclaration).solveSymbol(name, typeSolver) : typeDeclaration instanceof ReflectionInterfaceDeclaration ? ((ReflectionInterfaceDeclaration) typeDeclaration).solveSymbol(name, typeSolver) : typeDeclaration instanceof JavassistClassDeclaration ? ((JavassistClassDeclaration) typeDeclaration).solveSymbol(name, typeSolver) : SymbolReference.unsolved(ValueDeclaration.class);
    }
    @Deprecated
    public SymbolReference<TypeDeclaration> solveTypeInType(TypeDeclaration typeDeclaration, String name) {
        return typeDeclaration instanceof JavaParserClassDeclaration ? ((JavaParserClassDeclaration) typeDeclaration).solveType(name, typeSolver) : typeDeclaration instanceof JavaParserInterfaceDeclaration ? ((JavaParserInterfaceDeclaration) typeDeclaration).solveType(name, typeSolver) : SymbolReference.unsolved(ReferenceTypeDeclaration.class);
    }
}
