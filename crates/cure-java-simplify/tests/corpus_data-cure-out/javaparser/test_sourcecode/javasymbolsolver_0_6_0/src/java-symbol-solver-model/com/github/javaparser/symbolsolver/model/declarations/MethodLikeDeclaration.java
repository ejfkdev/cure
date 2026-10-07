package com.github.javaparser.symbolsolver.model.declarations;

import java.util.Optional;

public interface MethodLikeDeclaration extends Declaration, TypeParametrizable, HasAccessLevel {
    default String getPackageName() {
        return declaringType().getPackageName();
    }
    default String getClassName() {
        return declaringType().getClassName();
    }
    default String getQualifiedName() {
        return declaringType().getQualifiedName() + "." + this.getName();
    }
    default String getSignature() {
        StringBuffer sb = new StringBuffer();
        sb.append(getName());
        sb.append("(");
        for (int i = 0; i < getNumberOfParams(); i++) {
            if (i != 0) {
                sb.append(", ");
            }
            sb.append(getParam(i).describeType());
        }
        sb.append(")");
        return sb.toString();
    }
    default String getQualifiedSignature() {
        return declaringType().getId() + "." + this.getSignature();
    }
    ReferenceTypeDeclaration declaringType();
    int getNumberOfParams();
    ParameterDeclaration getParam(int i);
    default ParameterDeclaration getLastParam() {
        if (getNumberOfParams() == 0) {
            throw new UnsupportedOperationException("This method has no typeParametersValues, therefore it has no a last parameter");
        }
        return getParam(getNumberOfParams() - 1);
    }
    default boolean hasVariadicParameter() {
        return getNumberOfParams() == 0 ? false : getParam(getNumberOfParams() - 1).isVariadic();
    }
    @Override
    default Optional<TypeParameterDeclaration> findTypeParameter(String name) {
        for (TypeParameterDeclaration tp : this.getTypeParameters()) {
            if (tp.getName().equals(name)) {
                return Optional.of(tp);
            }
        }
        return declaringType().findTypeParameter(name);
    }
}
