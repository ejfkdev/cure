package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.*;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.javaparsermodel.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.logic.AbstractClassDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.declarations.ConstructorDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.FieldDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.LazyType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import com.google.common.collect.ImmutableList;
import java.util.*;
import java.util.stream.Collectors;

public class JavaParserClassDeclaration extends AbstractClassDeclaration {
    private TypeSolver typeSolver;
    private com.github.javaparser.ast.body.ClassOrInterfaceDeclaration wrappedNode;
    private JavaParserTypeAdapter<ClassOrInterfaceDeclaration> javaParserTypeAdapter;
    public JavaParserClassDeclaration(com.github.javaparser.ast.body.ClassOrInterfaceDeclaration wrappedNode, TypeSolver typeSolver) {
        if (wrappedNode.isInterface()) {
            throw new IllegalArgumentException("Interface given");
        }
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
        this.javaParserTypeAdapter = new JavaParserTypeAdapter<ClassOrInterfaceDeclaration>(wrappedNode, typeSolver);
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        JavaParserClassDeclaration that = (JavaParserClassDeclaration) o;
        return wrappedNode.equals(that.wrappedNode);
    }
    @Override
    public int hashCode() {
        return wrappedNode.hashCode();
    }
    @Override
    public String toString() {
        return "JavaParserClassDeclaration{wrappedNode=" + wrappedNode + '}';
    }
    @Override
    public List<FieldDeclaration> getAllFields() {
        ArrayList<FieldDeclaration> fields = new ArrayList<>();
        for (BodyDeclaration<?> member : wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.FieldDeclaration) {
                com.github.javaparser.ast.body.FieldDeclaration field = (com.github.javaparser.ast.body.FieldDeclaration) member;
                for (VariableDeclarator vd : field.getVariables()) {
                    fields.add(new JavaParserFieldDeclaration(vd, typeSolver));
                }
            }
        }
        ClassDeclaration superclass = (ClassDeclaration) this.getSuperClass().getTypeDeclaration();
        if (superclass != this) 
            fields.addAll(superclass.getAllFields());
        getInterfaces().forEach((interf) -> interf.getTypeDeclaration().getAllFields().forEach((f) -> {
            fields.add(f);
        }));
        return fields;
    }
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> parameterTypes) {
        return getContext().solveMethod(name, parameterTypes, false, typeSolver);
    }
    @Deprecated
    public Context getContext() {
        return JavaParserFactory.getContext(wrappedNode, typeSolver);
    }
    public Type getUsage(Node node) {
        throw new UnsupportedOperationException();
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    @Override
    public ReferenceType getSuperClass() {
        return wrappedNode.getExtendedTypes().isEmpty() ? object() : toReferenceType(wrappedNode.getExtendedTypes().get(0));
    }
    @Override
    public List<ReferenceType> getInterfaces() {
        List<ReferenceType> interfaces = new ArrayList<>();
        if (wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType t : wrappedNode.getImplementedTypes()) {
                interfaces.add(toReferenceType(t));
            }
        }
        return interfaces;
    }
    @Override
    public List<ConstructorDeclaration> getConstructors() {
        List<ConstructorDeclaration> declared = new LinkedList<>();
        for (BodyDeclaration<?> member : wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.ConstructorDeclaration) {
                com.github.javaparser.ast.body.ConstructorDeclaration constructorDeclaration = (com.github.javaparser.ast.body.ConstructorDeclaration) member;
                declared.add(new JavaParserConstructorDeclaration(this, constructorDeclaration, typeSolver));
            }
        }
        return declared.isEmpty() ? ImmutableList.of(new DefaultConstructorDeclaration(this)) : declared;
    }
    @Override
    public boolean hasDirectlyAnnotation(String canonicalName) {
        for (AnnotationExpr annotationExpr : wrappedNode.getAnnotations()) {
            if (solveType(annotationExpr.getName().getId(), typeSolver).getCorrespondingDeclaration().getQualifiedName().equals(canonicalName)) {
                return true;
            }
        }
        return false;
    }
    @Override
    public boolean isInterface() {
        return wrappedNode.isInterface();
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
    public boolean canBeAssignedTo(ReferenceTypeDeclaration other) {
        if (this.getQualifiedName().equals(other.getQualifiedName())) {
            return true;
        }
        ClassDeclaration superclass = (ClassDeclaration) getSuperClass().getTypeDeclaration();
        if (superclass != null) {
            if (Object.class.getCanonicalName().equals(superclass.getQualifiedName())) {
                return true;
            }
            if (superclass.canBeAssignedTo(other)) {
                return true;
            }
        }
        if (this.wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType type : wrappedNode.getImplementedTypes()) {
                ReferenceTypeDeclaration ancestor = (ReferenceTypeDeclaration) new SymbolSolver(typeSolver).solveType(type);
                if (ancestor.canBeAssignedTo(other)) {
                    return true;
                }
            }
        }
        return false;
    }
    @Override
    public boolean isTypeParameter() {
        return false;
    }
    @Deprecated
    public SymbolReference<TypeDeclaration> solveType(String name, TypeSolver typeSolver) {
        if (this.wrappedNode.getName().getId().equals(name)) {
            return SymbolReference.solved(this);
        }
        SymbolReference<TypeDeclaration> ref = javaParserTypeAdapter.solveType(name, typeSolver);
        if (ref.isSolved()) {
            return ref;
        }
        String prefix = wrappedNode.getName() + ".";
        return name.startsWith(prefix) && name.length() > prefix.length() ? new JavaParserClassDeclaration(this.wrappedNode, typeSolver).solveType(name.substring(prefix.length()), typeSolver) : getContext().getParent().solveType(name, typeSolver);
    }
    @Override
    public List<ReferenceType> getAncestors() {
        List<ReferenceType> ancestors = new ArrayList<>();
        ReferenceType superclass = getSuperClass();
        if (superclass != null) {
            ancestors.add(superclass);
        }
        if (wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType implemented : wrappedNode.getImplementedTypes()) {
                ReferenceType ancestor = toReferenceType(implemented);
                ancestors.add(ancestor);
            }
        }
        return ancestors;
    }
    @Override
    public Set<MethodDeclaration> getDeclaredMethods() {
        Set<MethodDeclaration> methods = new HashSet<>();
        for (BodyDeclaration<?> member : wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.MethodDeclaration) {
                methods.add(new JavaParserMethodDeclaration((com.github.javaparser.ast.body.MethodDeclaration) member, typeSolver));
            }
        }
        return methods;
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return this.wrappedNode.getTypeParameters().stream().map((tp) -> new JavaParserTypeParameter(tp, typeSolver)).collect(Collectors.toList());
    }
    public com.github.javaparser.ast.body.ClassOrInterfaceDeclaration getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public AccessLevel accessLevel() {
        return Helper.toAccessLevel(wrappedNode.getModifiers());
    }
    @Override
    protected ReferenceType object() {
        return new ReferenceTypeImpl(typeSolver.solveType(Object.class.getCanonicalName()), typeSolver);
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
    private ReferenceType toReferenceType(ClassOrInterfaceType classOrInterfaceType) {
        String className = classOrInterfaceType.getName().getId();
        if (classOrInterfaceType.getScope().isPresent()) {
            className = classOrInterfaceType.getScope().get().toString() + "." + className;
        }
        SymbolReference<TypeDeclaration> ref = solveType(className, typeSolver);
        if (!ref.isSolved()) {
            Optional<ClassOrInterfaceType> localScope = classOrInterfaceType.getScope();
            if (localScope.isPresent()) {
                ref = solveType(localScope.get().getName().getId() + "." + classOrInterfaceType.getName().getId(), typeSolver);
            }
        }
        if (!ref.isSolved()) {
            throw new UnsolvedSymbolException(classOrInterfaceType.getName().getId());
        }
        if (!classOrInterfaceType.getTypeArguments().isPresent()) {
            return new ReferenceTypeImpl(ref.getCorrespondingDeclaration().asReferenceType(), typeSolver);
        }
        List<Type> superClassTypeParameters = classOrInterfaceType.getTypeArguments().get().stream().map((ta) -> new LazyType((v) -> JavaParserFacade.get(typeSolver).convert(ta, ta))).collect(Collectors.toList());
        return new ReferenceTypeImpl(ref.getCorrespondingDeclaration().asReferenceType(), superClassTypeParameters, typeSolver);
    }
}
