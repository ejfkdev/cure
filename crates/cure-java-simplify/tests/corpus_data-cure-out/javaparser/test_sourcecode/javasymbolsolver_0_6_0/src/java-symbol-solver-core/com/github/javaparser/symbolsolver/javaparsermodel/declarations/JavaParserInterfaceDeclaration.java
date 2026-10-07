package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFactory;
import com.github.javaparser.symbolsolver.javaparsermodel.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.logic.AbstractTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.*;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.LazyType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceTypeImpl;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.resolution.SymbolSolver;
import java.util.*;
import java.util.stream.Collectors;

public class JavaParserInterfaceDeclaration extends AbstractTypeDeclaration implements InterfaceDeclaration {
    private TypeSolver typeSolver;
    private ClassOrInterfaceDeclaration wrappedNode;
    private JavaParserTypeAdapter javaParserTypeAdapter;
    public JavaParserInterfaceDeclaration(ClassOrInterfaceDeclaration wrappedNode, TypeSolver typeSolver) {
        if (!wrappedNode.isInterface()) {
            throw new IllegalArgumentException();
        }
        this.wrappedNode = wrappedNode;
        this.typeSolver = typeSolver;
        this.javaParserTypeAdapter = new JavaParserTypeAdapter(wrappedNode, typeSolver);
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
    public Type getUsage(Node node) {
        throw new UnsupportedOperationException();
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        JavaParserInterfaceDeclaration that = (JavaParserInterfaceDeclaration) o;
        return wrappedNode.equals(that.wrappedNode);
    }
    @Override
    public int hashCode() {
        return wrappedNode.hashCode();
    }
    @Override
    public String getName() {
        return wrappedNode.getName().getId();
    }
    @Override
    public InterfaceDeclaration asInterface() {
        return this;
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
        return true;
    }
    @Override
    public List<ReferenceType> getInterfacesExtended() {
        List<ReferenceType> interfaces = new ArrayList<>();
        if (wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType t : wrappedNode.getImplementedTypes()) {
                interfaces.add(new ReferenceTypeImpl(solveType(t.getName().getId(), typeSolver).getCorrespondingDeclaration().asInterface(), typeSolver));
            }
        }
        return interfaces;
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
        if (this.wrappedNode.getExtendedTypes() != null) {
            for (ClassOrInterfaceType type : wrappedNode.getExtendedTypes()) {
                ReferenceTypeDeclaration ancestor = (ReferenceTypeDeclaration) new SymbolSolver(typeSolver).solveType(type);
                if (ancestor.canBeAssignedTo(other)) {
                    return true;
                }
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
    @Override
    public List<FieldDeclaration> getAllFields() {
        ArrayList<FieldDeclaration> fields = new ArrayList<>();
        for (BodyDeclaration member : wrappedNode.getMembers()) {
            if (member instanceof com.github.javaparser.ast.body.FieldDeclaration) {
                com.github.javaparser.ast.body.FieldDeclaration field = (com.github.javaparser.ast.body.FieldDeclaration) member;
                for (VariableDeclarator vd : field.getVariables()) {
                    fields.add(new JavaParserFieldDeclaration(vd, typeSolver));
                }
            }
        }
        getAncestors().forEach((a) -> {
            if (a.getTypeDeclaration() != this) {
                fields.addAll(a.getTypeDeclaration().getAllFields());
            }
        });
        return fields;
    }
    @Override
    public String toString() {
        return "JavaParserInterfaceDeclaration{wrappedNode=" + wrappedNode + '}';
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
        return name.startsWith(prefix) && name.length() > prefix.length() ? new JavaParserInterfaceDeclaration(this.wrappedNode, typeSolver).solveType(name.substring(prefix.length()), typeSolver) : getContext().getParent().solveType(name, typeSolver);
    }
    @Override
    public List<ReferenceType> getAncestors() {
        List<ReferenceType> ancestors = new ArrayList<>();
        if (wrappedNode.getExtendedTypes() != null) {
            for (ClassOrInterfaceType extended : wrappedNode.getExtendedTypes()) {
                ancestors.add(toReferenceType(extended));
            }
        }
        if (wrappedNode.getImplementedTypes() != null) {
            for (ClassOrInterfaceType implemented : wrappedNode.getImplementedTypes()) {
                ancestors.add(toReferenceType(implemented));
            }
        }
        return ancestors;
    }
    @Override
    public List<TypeParameterDeclaration> getTypeParameters() {
        return this.wrappedNode.getTypeParameters() == null ? Collections.emptyList() : this.wrappedNode.getTypeParameters().stream().map((tp) -> new JavaParserTypeParameter(tp, typeSolver)).collect(Collectors.toList());
    }
    public ClassOrInterfaceDeclaration getWrappedNode() {
        return wrappedNode;
    }
    @Override
    public AccessLevel accessLevel() {
        return Helper.toAccessLevel(wrappedNode.getModifiers());
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
        SymbolReference<? extends TypeDeclaration> ref = null;
        if (classOrInterfaceType.toString().indexOf('.') > -1) {
            ref = typeSolver.tryToSolveType(classOrInterfaceType.toString());
        }
        if (ref == null || !ref.isSolved()) {
            ref = solveType(classOrInterfaceType.toString(), typeSolver);
        }
        if (!ref.isSolved()) {
            ref = solveType(classOrInterfaceType.getName().getId(), typeSolver);
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
