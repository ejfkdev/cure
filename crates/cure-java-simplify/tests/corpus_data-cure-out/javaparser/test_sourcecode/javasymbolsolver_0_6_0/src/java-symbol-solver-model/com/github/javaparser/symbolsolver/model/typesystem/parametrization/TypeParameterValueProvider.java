package com.github.javaparser.symbolsolver.model.typesystem.parametrization;

import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.Wildcard;
import java.util.Optional;

public interface TypeParameterValueProvider {
    Optional<Type> typeParamValue(TypeParameterDeclaration typeParameterDeclaration);
    default Type useThisTypeParametersOnTheGivenType(Type type) {
        if (type.isTypeVariable()) {
            TypeParameterDeclaration typeParameter = type.asTypeParameter();
            if (typeParameter.declaredOnType()) {
                Optional<Type> typeParam = typeParamValue(typeParameter);
                if (typeParam.isPresent()) {
                    type = typeParam.get();
                }
            }
        }
        if (type.isWildcard() && type.asWildcard().isBounded()) {
            return type.asWildcard().isExtends() ? Wildcard.extendsBound(useThisTypeParametersOnTheGivenType(type.asWildcard().getBoundedType())) : Wildcard.superBound(useThisTypeParametersOnTheGivenType(type.asWildcard().getBoundedType()));
        }
        if (type.isReferenceType()) {
            type = type.asReferenceType().transformTypeParameters((tp) -> useThisTypeParametersOnTheGivenType(tp));
        }
        return type;
    }
    Optional<Type> getGenericParameterByName(String name);
}
