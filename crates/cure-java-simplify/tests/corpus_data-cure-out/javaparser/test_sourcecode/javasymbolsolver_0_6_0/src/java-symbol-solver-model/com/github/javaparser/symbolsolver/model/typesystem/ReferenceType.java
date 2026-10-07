package com.github.javaparser.symbolsolver.model.typesystem;

import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration.Bound;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParameterValueProvider;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParametersMap;
import com.github.javaparser.symbolsolver.model.typesystem.parametrization.TypeParametrized;
import javaslang.Tuple2;
import java.util.*;
import java.util.stream.Collectors;

public abstract class ReferenceType implements Type, TypeParametrized, TypeParameterValueProvider {
    protected ReferenceTypeDeclaration typeDeclaration;
    protected TypeSolver typeSolver;
    protected TypeParametersMap typeParametersMap;
    public ReferenceType(ReferenceTypeDeclaration typeDeclaration, TypeSolver typeSolver) {
        this(typeDeclaration, deriveParams(typeDeclaration), typeSolver);
    }
    public ReferenceType(ReferenceTypeDeclaration typeDeclaration, List<Type> typeParameters, TypeSolver typeSolver) {
        if (typeSolver == null) {
            throw new IllegalArgumentException("typeSolver should not be null");
        }
        if (typeDeclaration.isTypeParameter()) {
            throw new IllegalArgumentException("You should use only Classes, Interfaces and enums");
        }
        if (typeParameters.size() > 0 && typeParameters.size() != typeDeclaration.getTypeParameters().size()) {
            throw new IllegalArgumentException(String.format("expected either zero type parameters or has many as defined in the declaration (%d). Found %d", typeDeclaration.getTypeParameters().size(), typeParameters.size()));
        }
        TypeParametersMap.Builder typeParametersMapBuilder = new TypeParametersMap.Builder();
        for (int i = 0; i < typeParameters.size(); i++) {
            typeParametersMapBuilder.setValue(typeDeclaration.getTypeParameters().get(i), typeParameters.get(i));
        }
        this.typeParametersMap = typeParametersMapBuilder.build();
        this.typeDeclaration = typeDeclaration;
        this.typeSolver = typeSolver;
    }
    @Override
    public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        ReferenceType that = (ReferenceType) o;
        return !typeDeclaration.equals(that.typeDeclaration) ? false : typeParametersMap.equals(that.typeParametersMap);
    }
    @Override
    public int hashCode() {
        int result = typeDeclaration.hashCode();
        result = 31 * result + typeParametersMap.hashCode();
        return result;
    }
    @Override
    public String toString() {
        return "ReferenceType{" + getQualifiedName() + ", typeParametersMap=" + typeParametersMap + '}';
    }
    @Override
    public final boolean isReferenceType() {
        return true;
    }
    @Override
    public ReferenceType asReferenceType() {
        return this;
    }
    @Override
    public String describe() {
        StringBuilder sb = new StringBuilder();
        if (hasName()) {
            sb.append(typeDeclaration.getQualifiedName());
        } else {
            sb.append("<anonymous class>");
        }
        if (!typeParametersMap().isEmpty()) {
            sb.append("<");
            sb.append(String.join(", ", typeDeclaration.getTypeParameters().stream().map((tp) -> typeParametersMap().getValue(tp).describe()).collect(Collectors.toList())));
            sb.append(">");
        }
        return sb.toString();
    }
    public Type transformTypeParameters(TypeTransformer transformer) {
        Type result = this;
        int i = 0;
        for (Type tp : this.typeParametersValues()) {
            Type transformedTp = transformer.transform(tp);
            if (transformedTp != tp) {
                List<Type> typeParametersCorrected = result.asReferenceType().typeParametersValues();
                typeParametersCorrected.set(i, transformedTp);
                result = create(typeDeclaration, typeParametersCorrected, typeSolver);
            }
            i++;
        }
        return result;
    }
    @Override
    public Type replaceTypeVariables(TypeParameterDeclaration tpToReplace, Type replaced, Map<TypeParameterDeclaration, Type> inferredTypes) {
        if (replaced == null) {
            throw new IllegalArgumentException();
        }
        ReferenceType result = this;
        int i = 0;
        for (Type tp : this.typeParametersValues()) {
            Type transformedTp = tp.replaceTypeVariables(tpToReplace, replaced, inferredTypes);
            if (tp.isTypeVariable() && tp.asTypeVariable().describe().equals(tpToReplace.getName())) {
                inferredTypes.put(tp.asTypeParameter(), replaced);
            }
            List<Type> typeParametersCorrected = result.asReferenceType().typeParametersValues();
            typeParametersCorrected.set(i, transformedTp);
            result = create(typeDeclaration, typeParametersCorrected, typeSolver);
            i++;
        }
        List<Type> values = result.typeParametersValues();
        if (values.contains(tpToReplace)) {
            int index = values.indexOf(tpToReplace);
            values.set(index, replaced);
            return create(result.getTypeDeclaration(), values, typeSolver);
        }
        return result;
    }
    @Override
    public abstract boolean isAssignableBy(Type other);
    public List<ReferenceType> getAllAncestors() {
        List<ReferenceType> ancestors = typeDeclaration.getAllAncestors();
        ancestors = ancestors.stream().map((a) -> typeParametersMap().replaceAll(a).asReferenceType()).collect(Collectors.toList());
        ancestors.removeIf((a) -> a.getQualifiedName().equals(Object.class.getCanonicalName()));
        ReferenceType objectRef = create(typeSolver.solveType(Object.class.getCanonicalName()), typeSolver);
        ancestors.add(objectRef);
        return ancestors;
    }
    public final List<ReferenceType> getAllInterfacesAncestors() {
        return getAllAncestors().stream().filter((it) -> it.getTypeDeclaration().isInterface()).collect(Collectors.toList());
    }
    public final List<ReferenceType> getAllClassesAncestors() {
        return getAllAncestors().stream().filter((it) -> it.getTypeDeclaration().isClass()).collect(Collectors.toList());
    }
    public Optional<Type> getGenericParameterByName(String name) {
        for (TypeParameterDeclaration tp : typeDeclaration.getTypeParameters()) {
            if (tp.getName().equals(name)) {
                return Optional.of(this.typeParametersMap().getValue(tp));
            }
        }
        return Optional.empty();
    }
    public List<Type> typeParametersValues() {
        return this.typeParametersMap.isEmpty() ? Collections.emptyList() : typeDeclaration.getTypeParameters().stream().map((tp) -> typeParametersMap.getValue(tp)).collect(Collectors.toList());
    }
    public List<Tuple2<TypeParameterDeclaration, Type>> getTypeParametersMap() {
        List<Tuple2<TypeParameterDeclaration, Type>> typeParametersMap = new ArrayList<>();
        if (!isRawType()) {
            for (int i = 0; i < typeDeclaration.getTypeParameters().size(); i++) {
                typeParametersMap.add(new Tuple2<>(typeDeclaration.getTypeParameters().get(0), typeParametersValues().get(i)));
            }
        }
        return typeParametersMap;
    }
    @Override
    public TypeParametersMap typeParametersMap() {
        return typeParametersMap;
    }
    public final ReferenceTypeDeclaration getTypeDeclaration() {
        return typeDeclaration;
    }
    public Optional<Type> getFieldType(String name) {
        if (!typeDeclaration.hasField(name)) {
            return Optional.empty();
        }
        Type type = typeDeclaration.getField(name).getType();
        type = useThisTypeParametersOnTheGivenType(type);
        return Optional.of(type);
    }
    public boolean hasName() {
        return typeDeclaration.hasName();
    }
    public String getQualifiedName() {
        return typeDeclaration.getQualifiedName();
    }
    public String getId() {
        return typeDeclaration.getId();
    }
    public abstract Set<MethodUsage> getDeclaredMethods();
    public boolean isRawType() {
        if (!typeDeclaration.getTypeParameters().isEmpty()) {
            if (typeParametersMap().isEmpty()) {
                return true;
            }
            for (String name : typeParametersMap().getNames()) {
                Optional<Type> value = typeParametersMap().getValueBySignature(name);
                if (!(value.isPresent() && value.get().isTypeVariable() && value.get().asTypeVariable().qualifiedName().equals(name))) {
                    return false;
                }
            }
            return true;
        }
        return false;
    }
    public Optional<Type> typeParamValue(TypeParameterDeclaration typeParameterDeclaration) {
        if (typeParameterDeclaration.declaredOnMethod()) {
            throw new IllegalArgumentException();
        }
        String typeId = this.getTypeDeclaration().getId();
        if (typeId.equals(typeParameterDeclaration.getContainerId())) {
            return Optional.of(this.typeParametersMap().getValue(typeParameterDeclaration));
        }
        for (ReferenceType ancestor : this.getAllAncestors()) {
            if (ancestor.getId().equals(typeParameterDeclaration.getContainerId())) {
                return Optional.of(ancestor.typeParametersMap().getValue(typeParameterDeclaration));
            }
        }
        return Optional.empty();
    }
    protected abstract ReferenceType create(ReferenceTypeDeclaration typeDeclaration, List<Type> typeParameters, TypeSolver typeSolver);
    protected ReferenceType create(ReferenceTypeDeclaration typeDeclaration, TypeParametersMap typeParametersMap, TypeSolver typeSolver) {
        return create(typeDeclaration, typeDeclaration.getTypeParameters().stream().map((tp) -> typeParametersMap.getValue(tp)).collect(Collectors.toList()), typeSolver);
    }
    protected abstract ReferenceType create(ReferenceTypeDeclaration typeDeclaration, TypeSolver typeSolver);
    protected boolean isCorrespondingBoxingType(String typeName) {
        switch (typeName) {
            case "boolean":
                return getQualifiedName().equals(Boolean.class.getCanonicalName());
            case "char":
                return getQualifiedName().equals(Character.class.getCanonicalName());
            case "byte":
                return getQualifiedName().equals(Byte.class.getCanonicalName());
            case "short":
                return getQualifiedName().equals(Short.class.getCanonicalName());
            case "int":
                return getQualifiedName().equals(Integer.class.getCanonicalName());
            case "long":
                return getQualifiedName().equals(Long.class.getCanonicalName());
            case "float":
                return getQualifiedName().equals(Float.class.getCanonicalName());
            case "double":
                return getQualifiedName().equals(Double.class.getCanonicalName());
            default:
                throw new UnsupportedOperationException(typeName);
        }
    }
    protected boolean compareConsideringTypeParameters(ReferenceType other) {
        if (other.equals(this)) {
            return true;
        }
        if (this.getQualifiedName().equals(other.getQualifiedName())) {
            if (this.isRawType() || other.isRawType()) {
                return true;
            }
            if (this.typeParametersValues().size() != other.typeParametersValues().size()) {
                throw new IllegalStateException();
            }
            for (int i = 0; i < typeParametersValues().size(); i++) {
                Type thisParam = typeParametersValues().get(i);
                Type otherParam = other.typeParametersValues().get(i);
                if (!thisParam.equals(otherParam)) {
                    if (thisParam instanceof Wildcard) {
                        Wildcard thisParamAsWildcard = (Wildcard) thisParam;
                        if (!(thisParamAsWildcard.isSuper() && otherParam.isAssignableBy(thisParamAsWildcard.getBoundedType()))) 
                            if (!(thisParamAsWildcard.isExtends() && thisParamAsWildcard.getBoundedType().isAssignableBy(otherParam))) 
                                if (thisParamAsWildcard.isBounded()) {
                                    return false;
                                }
                    } else {
                        if (thisParam instanceof TypeVariable && otherParam instanceof TypeVariable) {
                            List<Type> thisBounds = thisParam.asTypeVariable().asTypeParameter().getBounds(this.typeSolver).stream().map((bound) -> bound.getType()).collect(Collectors.toList());
                            List<Type> otherBounds = otherParam.asTypeVariable().asTypeParameter().getBounds(other.typeSolver).stream().map((bound) -> bound.getType()).collect(Collectors.toList());
                            if (thisBounds.size() == otherBounds.size() && otherBounds.containsAll(thisBounds)) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            }
            return true;
        }
        return false;
    }
    private static List<Type> deriveParams(ReferenceTypeDeclaration typeDeclaration) {
        return typeDeclaration.getTypeParameters().stream().map((tp) -> new TypeVariable(tp)).collect(Collectors.toList());
    }
    public ReferenceType deriveTypeParameters(TypeParametersMap typeParametersMap) {
        return create(typeDeclaration, typeParametersMap, typeSolver);
    }
}
