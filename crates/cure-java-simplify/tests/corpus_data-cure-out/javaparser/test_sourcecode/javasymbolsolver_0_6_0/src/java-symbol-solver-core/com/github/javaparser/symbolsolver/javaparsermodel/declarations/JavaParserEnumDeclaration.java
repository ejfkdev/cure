package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.body.EnumConstantDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.javaparsermodel.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.logic.AbstractTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.ArrayType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParametersMap;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionFactory;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import java.io.Serializable;
import java.util.*;

public class JavaParserEnumDeclaration extends AbstractTypeDeclaration implements EnumDeclaration {
    private TypeSolver typeSolver;
    private com.github.javaparser.ast.body.EnumDeclaration wrappedNode;
    private JavaParserTypeAdapter javaParserTypeAdapter;
    public JavaParserEnumDeclaration(com.github.javaparser.ast.body.EnumDeclaration wrappedNode, TypeSolver typeSolver) {
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
        this.javaParserTypeAdapter = new JavaParserTypeAdapter(wrappedNode, typeSolver);
    }
    @Override
    public String toString() {
        return "JavaParserEnumDeclaration{wrappedNode=" + wrappedNode + '}';
    }
    @Override
    public Set<MethodDeclaration> getDeclaredMethods() {
        Set<MethodDeclaration> methods = new HashSet<>();
        for (BodyDeclaration member : wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.MethodDeclaration) {
                methods.add(new JavaParserMethodDeclaration((com.github.javaparser.ast.body.MethodDeclaration) member, typeSolver));
            }
        }
        return methods;
    }
    public Context getContext() {
        return JavaParserFactory.getContext(wrappedNode, typeSolver);
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
        return false;
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
    public boolean canBeAssignedTo(ReferenceTypeDeclaration other) {
        return other.getQualifiedName().equals(this.getQualifiedName()) || (other.getQualifiedName().equals(Enum.class.getCanonicalName()) || (other.getQualifiedName().equals(Comparable.class.getCanonicalName()) || (other.getQualifiedName().equals(Serializable.class.getCanonicalName()) || other.getQualifiedName().equals(Object.class.getCanonicalName()))));
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
    public String getPackageName() {
        return javaParserTypeAdapter.getPackageName();
    }
    @Override
    public String getClassName() {
        return javaParserTypeAdapter.getClassName();
    }
    @Override
    public String getQualifiedName() {
        return javaParserTypeAdapter.getQualifiedName();
    }
    @Override
    public boolean isAssignableBy(ReferenceTypeDeclaration other) {
        return javaParserTypeAdapter.isAssignableBy(other);
    }
    @Override
    public boolean isAssignableBy(Type type) {
        return javaParserTypeAdapter.isAssignableBy(type);
    }
    @Override
    public boolean isTypeParameter() {
        return false;
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        JavaParserEnumDeclaration that = (JavaParserEnumDeclaration) o;
        return wrappedNode.equals(that.wrappedNode);
    }
    @Override
    public int hashCode() {
        return wrappedNode.hashCode();
    }
    @Deprecated
    public Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> parameterTypes, TypeSolver typeSolver, Context invokationContext, List<Type> typeParameterValues) {
        return name.equals("values") && parameterTypes.isEmpty() ? Optional.of(new ValuesMethod(this, typeSolver).getUsage(null)) : getContext().solveMethodAsUsage(name, parameterTypes, typeSolver);
    }
    @Override
    public List<FieldDeclaration> getAllFields() {
        ArrayList<FieldDeclaration> fields = new ArrayList<>();
        if (this.wrappedNode.getMembers() != null) {
            for (BodyDeclaration member : this.wrappedNode.getMembers()) {
                if (member instanceof com.github.javaparser.ast.body.FieldDeclaration) {
                    com.github.javaparser.ast.body.FieldDeclaration field = (com.github.javaparser.ast.body.FieldDeclaration) member;
                    for (VariableDeclarator vd : field.getVariables()) {
                        fields.add(new JavaParserFieldDeclaration(vd, typeSolver));
                    }
                }
            }
        }
        if (this.wrappedNode.getEntries() != null) {
            for (EnumConstantDeclaration member : this.wrappedNode.getEntries()) {
                fields.add(new JavaParserFieldDeclaration(member, typeSolver));
            }
        }
        return fields;
    }
    @Override
    public List<ReferenceType> getAncestors() {
        List<ReferenceType> ancestors = new ArrayList<>();
        ReferenceType enumClass = ReflectionFactory.typeUsageFor(Enum.class, typeSolver).asReferenceType();
        TypeParameterDeclaration eTypeParameter = enumClass.getTypeDeclaration().getTypeParameters().get(0);
        enumClass = enumClass.deriveTypeParameters(new TypeParametersMap.Builder().setValue(eTypeParameter, new ReferenceTypeImpl(this, typeSolver)).build());
        ancestors.add(enumClass);
        if (wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType implementedType : wrappedNode.getImplementedTypes()) {
                SymbolReference<TypeDeclaration> implementedDeclRef = new SymbolSolver(typeSolver).solveTypeInType(this, implementedType.getName().getId());
                if (!implementedDeclRef.isSolved()) {
                    throw new UnsolvedSymbolException(implementedType.getName().getId());
                }
                ancestors.add(new ReferenceTypeImpl((ReferenceTypeDeclaration) implementedDeclRef.getCorrespondingDeclaration(), typeSolver));
            }
        }
        return ancestors;
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return Collections.emptyList();
    }
    public com.github.javaparser.ast.body.EnumDeclaration getWrappedNode() {
        return wrappedNode;
    }
    public static class ValuesMethod implements MethodDeclaration {
        private JavaParserEnumDeclaration enumDeclaration;
        private TypeSolver typeSolver;
        public ValuesMethod(JavaParserEnumDeclaration enumDeclaration, TypeSolver typeSolver) {
            this.enumDeclaration = enumDeclaration;
            this.typeSolver = typeSolver;
        }
        @Override
        public ReferenceTypeDeclaration declaringType() {
            return enumDeclaration;
        }
        @Override
        public Type getReturnType() {
            return new ArrayType(new ReferenceTypeImpl(enumDeclaration, typeSolver));
        }
        @Override
        public int getNumberOfParams() {
            return 0;
        }
        @Override
        public ParameterDeclaration getParam(int i) {
            throw new UnsupportedOperationException();
        }
        public MethodUsage getUsage(Node node) {
            throw new UnsupportedOperationException();
        }
        public MethodUsage resolveTypeVariables(Context context, List<Type> parameterTypes) {
            return new MethodUsage(this);
        }
        @Override
        public boolean isAbstract() {
            throw new UnsupportedOperationException();
        }
        @Override
        public boolean isDefaultMethod() {
            return false;
        }
        @Override
        public boolean isStatic() {
            return false;
        }
        @Override
        public String getName() {
            return "values";
        }
        @Override
        public List<TypeParameterDeclaration> getTypeParameters() {
            return Collections.emptyList();
        }
        @Override
        public AccessLevel accessLevel() {
            return Helper.toAccessLevel(enumDeclaration.getWrappedNode().getModifiers());
        }
    }
    @Override
    public AccessLevel accessLevel() {
        throw new UnsupportedOperationException();
    }
    @Override
    public Set<ReferenceTypeDeclaration> internalTypes() {
        Set<ReferenceTypeDeclaration> res = new HashSet<>();
        for (BodyDeclaration member : this.wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.TypeDeclaration) {
                res.add(JavaParserFacade.get(typeSolver).getTypeDeclaration((com.github.javaparser.ast.body.TypeDeclaration) member));
            }
        }
        return res;
    }
    @Override
    public Optional<ReferenceTypeDeclaration> containerType() {
        return javaParserTypeAdapter.containerType();
    }
}
