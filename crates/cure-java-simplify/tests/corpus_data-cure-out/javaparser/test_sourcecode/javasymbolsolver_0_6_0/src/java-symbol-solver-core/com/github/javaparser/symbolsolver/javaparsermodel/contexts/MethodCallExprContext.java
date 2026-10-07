package com.github.javaparser.symbolsolver.javaparsermodel.contexts;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.javaparsermodel.JavaParserFacade;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import com.github.javaparser.symbolsolver.model.resolution.Value;
import com.github.javaparser.symbolsolver.model.typesystem.*;
import com.github.javaparser.symbolsolver.model.typesystem.ArrayType;
import com.github.javaparser.symbolsolver.model.typesystem.ReferenceType;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.reflectionmodel.ReflectionClassDeclaration;
import com.github.javaparser.symbolsolver.resolution.MethodResolutionLogic;
import javaslang.Tuple2;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public class MethodCallExprContext extends AbstractJavaParserContext<MethodCallExpr> {
    public MethodCallExprContext(MethodCallExpr wrappedNode, TypeSolver typeSolver) {
        super(wrappedNode, typeSolver);
    }
    @Override
    public Optional<Type> solveGenericType(String name, TypeSolver typeSolver) {
        return wrappedNode.getScope().isPresent() ? JavaParserFacade.get(typeSolver).getType(wrappedNode.getScope().get()).asReferenceType().getGenericParameterByName(name) : Optional.empty();
    }
    @Override
    public String toString() {
        return "MethodCallExprContext{wrapped=" + wrappedNode + "}";
    }
    @Override
    public Optional<MethodUsage> solveMethodAsUsage(String name, List<Type> argumentsTypes, TypeSolver typeSolver) {
        if (wrappedNode.getScope().isPresent()) {
            Expression scope = wrappedNode.getScope().get();
            if (scope instanceof NameExpr) {
                SymbolReference<TypeDeclaration> ref = solveType(((NameExpr) scope).getName().getId(), typeSolver);
                if (ref.isSolved()) {
                    SymbolReference<MethodDeclaration> m = MethodResolutionLogic.solveMethodInType(ref.getCorrespondingDeclaration(), name, argumentsTypes, typeSolver);
                    if (m.isSolved()) {
                        MethodUsage methodUsage = new MethodUsage(m.getCorrespondingDeclaration());
                        methodUsage = resolveMethodTypeParametersFromExplicitList(typeSolver, methodUsage);
                        methodUsage = resolveMethodTypeParameters(methodUsage, argumentsTypes);
                        return Optional.of(methodUsage);
                    } else {
                        throw new UnsolvedSymbolException(ref.getCorrespondingDeclaration().toString(), "Method '" + name + "' with parameterTypes " + argumentsTypes);
                    }
                }
            }
            Type typeOfScope = JavaParserFacade.get(typeSolver).getType(scope);
            Map<TypeParameterDeclaration, Type> inferredTypes = new HashMap<>();
            for (int i = 0; i < argumentsTypes.size(); i++) {
                Type updatedArgumentType = usingParameterTypesFromScope(typeOfScope, argumentsTypes.get(i), inferredTypes);
                argumentsTypes.set(i, updatedArgumentType);
            }
            for (int i = 0; i < argumentsTypes.size(); i++) {
                Type updatedArgumentType = applyInferredTypes(argumentsTypes.get(i), inferredTypes);
                argumentsTypes.set(i, updatedArgumentType);
            }
            return solveMethodAsUsage(typeOfScope, name, argumentsTypes, typeSolver, this);
        } else {
            Context parentContext = getParent();
            while (parentContext instanceof MethodCallExprContext) {
                parentContext = parentContext.getParent();
            }
            return parentContext.solveMethodAsUsage(name, argumentsTypes, typeSolver);
        }
    }
    private MethodUsage resolveMethodTypeParametersFromExplicitList(TypeSolver typeSolver, MethodUsage methodUsage) {
        if (wrappedNode.getTypeArguments().isPresent()) {
            List<Type> typeArguments = new ArrayList<>();
            for (com.github.javaparser.ast.type.Type ty : wrappedNode.getTypeArguments().get()) {
                typeArguments.add(JavaParserFacade.get(typeSolver).convertToUsage(ty));
            }
            List<TypeParameterDeclaration> tyParamDecls = methodUsage.getDeclaration().getTypeParameters();
            if (tyParamDecls.size() == typeArguments.size()) {
                for (int i = 0; i < tyParamDecls.size(); i++) {
                    methodUsage = methodUsage.replaceTypeParameter(tyParamDecls.get(i), typeArguments.get(i));
                }
            }
        }
        return methodUsage;
    }
    @Override
    public SymbolReference<? extends ValueDeclaration> solveSymbol(String name, TypeSolver typeSolver) {
        return getParent().solveSymbol(name, typeSolver);
    }
    @Override
    public Optional<Value> solveSymbolAsValue(String name, TypeSolver typeSolver) {
        return getParent().solveSymbolAsValue(name, typeSolver);
    }
    @Override
    public SymbolReference<MethodDeclaration> solveMethod(String name, List<Type> argumentsTypes, boolean staticOnly, TypeSolver typeSolver) {
        if (wrappedNode.getScope().isPresent()) {
            Expression scope = wrappedNode.getScope().get();
            if (scope instanceof NameExpr) {
                NameExpr scopeAsName = (NameExpr) scope;
                SymbolReference<TypeDeclaration> symbolReference = this.solveType(scopeAsName.getName().getId(), typeSolver);
                if (symbolReference.isSolved() && symbolReference.getCorrespondingDeclaration().isType()) {
                    TypeDeclaration typeDeclaration = symbolReference.getCorrespondingDeclaration().asType();
                    return MethodResolutionLogic.solveMethodInType(typeDeclaration, name, argumentsTypes, false, typeSolver);
                }
            }
            Type typeOfScope = null;
            try {
                typeOfScope = JavaParserFacade.get(typeSolver).getType(scope);
            } catch (Exception e) {
                throw new RuntimeException(String.format("Issue calculating the type of the scope of " + this), e);
            }
            if (typeOfScope.isWildcard()) {
                return typeOfScope.asWildcard().isExtends() || typeOfScope.asWildcard().isSuper() ? MethodResolutionLogic.solveMethodInType(typeOfScope.asWildcard().getBoundedType().asReferenceType().getTypeDeclaration(), name, argumentsTypes, false, typeSolver) : MethodResolutionLogic.solveMethodInType(new ReflectionClassDeclaration(Object.class, typeSolver), name, argumentsTypes, false, typeSolver);
            } else if (typeOfScope.isArray()) {
                return MethodResolutionLogic.solveMethodInType(new ReflectionClassDeclaration(Object.class, typeSolver), name, argumentsTypes, false, typeSolver);
            } else if (typeOfScope.isTypeVariable()) {
                for (TypeParameterDeclaration.Bound bound : typeOfScope.asTypeParameter().getBounds(typeSolver)) {
                    SymbolReference<MethodDeclaration> res = MethodResolutionLogic.solveMethodInType(bound.getType().asReferenceType().getTypeDeclaration(), name, argumentsTypes, false, typeSolver);
                    if (res.isSolved()) {
                        return res;
                    }
                }
                return SymbolReference.unsolved(MethodDeclaration.class);
            } else 
                return typeOfScope.isConstraint() ? MethodResolutionLogic.solveMethodInType(typeOfScope.asConstraintType().getBound().asReferenceType().getTypeDeclaration(), name, argumentsTypes, typeSolver) : MethodResolutionLogic.solveMethodInType(typeOfScope.asReferenceType().getTypeDeclaration(), name, argumentsTypes, false, typeSolver);
        } else {
            Type typeOfScope = JavaParserFacade.get(typeSolver).getTypeOfThisIn(wrappedNode);
            return MethodResolutionLogic.solveMethodInType(typeOfScope.asReferenceType().getTypeDeclaration(), name, argumentsTypes, false, typeSolver);
        }
    }
    private Optional<MethodUsage> solveMethodAsUsage(ReferenceType refType, String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext) {
        Optional<MethodUsage> ref = ContextHelper.solveMethodAsUsage(refType.getTypeDeclaration(), name, argumentsTypes, typeSolver, invokationContext, refType.typeParametersValues());
        if (ref.isPresent()) {
            MethodUsage methodUsage = ref.get();
            methodUsage = resolveMethodTypeParametersFromExplicitList(typeSolver, methodUsage);
            Map<TypeParameterDeclaration, Type> derivedValues = new HashMap<>();
            for (int i = 0; i < methodUsage.getParamTypes().size(); i++) {
                ParameterDeclaration parameter = methodUsage.getDeclaration().getParam(i);
                Type parameterType = parameter.getType();
                if (parameter.isVariadic()) {
                    parameterType = parameterType.asArrayType().getComponentType();
                }
                inferTypes(argumentsTypes.get(i), parameterType, derivedValues);
            }
            for (Map.Entry<TypeParameterDeclaration, Type> entry : derivedValues.entrySet()) {
                methodUsage = methodUsage.replaceTypeParameter(entry.getKey(), entry.getValue());
            }
            Type returnType = refType.useThisTypeParametersOnTheGivenType(methodUsage.returnType());
            if (returnType != methodUsage.returnType()) {
                methodUsage = methodUsage.replaceReturnType(returnType);
            }
            for (int i = 0; i < methodUsage.getParamTypes().size(); i++) {
                Type replaced = refType.useThisTypeParametersOnTheGivenType(methodUsage.getParamTypes().get(i));
                methodUsage = methodUsage.replaceParamType(i, replaced);
            }
            return Optional.of(methodUsage);
        } else {
            return ref;
        }
    }
    private void inferTypes(Type source, Type target, Map<TypeParameterDeclaration, Type> mappings) {
        if (source.equals(target)) {
            return;
        }
        if (source.isReferenceType() && target.isReferenceType()) {
            ReferenceType sourceRefType = source.asReferenceType();
            ReferenceType targetRefType = target.asReferenceType();
            if (sourceRefType.getQualifiedName().equals(targetRefType.getQualifiedName())) {
                if (!sourceRefType.isRawType() && !targetRefType.isRawType()) {
                    for (int i = 0; i < sourceRefType.typeParametersValues().size(); i++) {
                        inferTypes(sourceRefType.typeParametersValues().get(i), targetRefType.typeParametersValues().get(i), mappings);
                    }
                }
            }
            return;
        }
        if (source.isReferenceType() && target.isWildcard()) {
            if (target.asWildcard().isBounded()) {
                inferTypes(source, target.asWildcard().getBoundedType(), mappings);
                return;
            }
            return;
        }
        if (source.isWildcard() && target.isWildcard()) {
            if (source.asWildcard().isBounded() && target.asWildcard().isBounded()) {
                inferTypes(source.asWildcard().getBoundedType(), target.asWildcard().getBoundedType(), mappings);
            }
            return;
        }
        if (source.isReferenceType() && target.isTypeVariable()) {
            mappings.put(target.asTypeParameter(), source);
            return;
        }
        if (source.isWildcard() && target.isTypeVariable()) {
            mappings.put(target.asTypeParameter(), source);
            return;
        }
        if (source.isArray() && target.isWildcard()) {
            if (target.asWildcard().isBounded()) {
                inferTypes(source, target.asWildcard().getBoundedType(), mappings);
                return;
            }
            return;
        }
        if (source.isArray() && target.isTypeVariable()) {
            mappings.put(target.asTypeParameter(), source);
            return;
        }
        if (source.isWildcard() && target.isReferenceType()) {
            if (source.asWildcard().isBounded()) {
                inferTypes(source.asWildcard().getBoundedType(), target, mappings);
            }
            return;
        }
        if (source.isConstraint() && target.isReferenceType()) {
            inferTypes(source.asConstraintType().getBound(), target, mappings);
            return;
        }
        if (source.isConstraint() && target.isTypeVariable()) {
            inferTypes(source.asConstraintType().getBound(), target, mappings);
            return;
        }
        if (source.isTypeVariable() && target.isTypeVariable()) {
            mappings.put(target.asTypeParameter(), source);
            return;
        }
        if (source.isPrimitive() || target.isPrimitive()) {
            return;
        }
        if (source.isNull()) {
            return;
        }
        throw new RuntimeException(source.describe() + " " + target.describe());
    }
    private MethodUsage resolveMethodTypeParameters(MethodUsage methodUsage, List<Type> actualParamTypes) {
        Map<TypeParameterDeclaration, Type> matchedTypeParameters = new HashMap<>();
        if (methodUsage.getDeclaration().hasVariadicParameter()) {
            if (actualParamTypes.size() == methodUsage.getDeclaration().getNumberOfParams()) {
                Type expectedType = methodUsage.getDeclaration().getLastParam().getType().asArrayType().getComponentType();
                Type actualType = actualParamTypes.get(actualParamTypes.size() - 1).isArray() ? actualParamTypes.get(actualParamTypes.size() - 1).asArrayType().getComponentType() : actualParamTypes.get(actualParamTypes.size() - 1);
                if (!expectedType.isAssignableBy(actualType)) {
                    for (TypeParameterDeclaration tp : methodUsage.getDeclaration().getTypeParameters()) {
                        expectedType = MethodResolutionLogic.replaceTypeParam(expectedType, tp, typeSolver);
                    }
                }
                if (!expectedType.isAssignableBy(actualType)) {
                    throw new UnsupportedOperationException(String.format("Unable to resolve the type typeParametersValues in a MethodUsage. Expected type: %s, Actual type: %s. Method Declaration: %s. MethodUsage: %s", expectedType, actualType, methodUsage.getDeclaration(), methodUsage));
                }
                matchTypeParameters(expectedType, actualType, matchedTypeParameters);
            } else {
                return methodUsage;
            }
        }
        int until = methodUsage.getDeclaration().hasVariadicParameter() ? actualParamTypes.size() - 1 : actualParamTypes.size();
        for (int i = 0; i < until; i++) {
            matchTypeParameters(methodUsage.getParamType(i), actualParamTypes.get(i), matchedTypeParameters);
        }
        for (TypeParameterDeclaration tp : matchedTypeParameters.keySet()) {
            methodUsage = methodUsage.replaceTypeParameter(tp, matchedTypeParameters.get(tp));
        }
        return methodUsage;
    }
    private void matchTypeParameters(Type expectedType, Type actualType, Map<TypeParameterDeclaration, Type> matchedTypeParameters) {
        if (expectedType.isTypeVariable()) {
            if (!actualType.isTypeVariable() && !actualType.isReferenceType()) {
                throw new UnsupportedOperationException(actualType.getClass().getCanonicalName());
            }
            matchedTypeParameters.put(expectedType.asTypeParameter(), actualType);
        } else if (expectedType.isArray()) {
            if (!actualType.isArray()) {
                throw new UnsupportedOperationException(actualType.getClass().getCanonicalName());
            }
            matchTypeParameters(expectedType.asArrayType().getComponentType(), actualType.asArrayType().getComponentType(), matchedTypeParameters);
        } else if (expectedType.isReferenceType()) {
            if (actualType.asReferenceType().typeParametersValues().size() > 0) {
                int i = 0;
                for (Type tp : expectedType.asReferenceType().typeParametersValues()) {
                    matchTypeParameters(tp, actualType.asReferenceType().typeParametersValues().get(i), matchedTypeParameters);
                    i++;
                }
            }
        } else if (!expectedType.isPrimitive()) 
            if (!expectedType.isWildcard()) {
                throw new UnsupportedOperationException(expectedType.getClass().getCanonicalName());
            }
    }
    private Optional<MethodUsage> solveMethodAsUsage(TypeVariable tp, String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext) {
        for (TypeParameterDeclaration.Bound bound : tp.asTypeParameter().getBounds(typeSolver)) {
            Optional<MethodUsage> methodUsage = solveMethodAsUsage(bound.getType(), name, argumentsTypes, typeSolver, invokationContext);
            if (methodUsage.isPresent()) {
                return methodUsage;
            }
        }
        return Optional.empty();
    }
    private Optional<MethodUsage> solveMethodAsUsage(Type type, String name, List<Type> argumentsTypes, TypeSolver typeSolver, Context invokationContext) {
        if (type instanceof ReferenceType) {
            return solveMethodAsUsage((ReferenceType) type, name, argumentsTypes, typeSolver, invokationContext);
        } else if (type instanceof TypeVariable) {
            return solveMethodAsUsage((TypeVariable) type, name, argumentsTypes, typeSolver, invokationContext);
        } else if (type instanceof Wildcard) {
            Wildcard wildcardUsage = (Wildcard) type;
            if (wildcardUsage.isSuper()) {
                return solveMethodAsUsage(wildcardUsage.getBoundedType(), name, argumentsTypes, typeSolver, invokationContext);
            } else if (wildcardUsage.isExtends()) {
                throw new UnsupportedOperationException("extends wildcard");
            } else {
                throw new UnsupportedOperationException("unbounded wildcard");
            }
        } else if (type instanceof LambdaConstraintType) {
            return solveMethodAsUsage(((LambdaConstraintType) type).getBound(), name, argumentsTypes, typeSolver, invokationContext);
        } else if (type instanceof ArrayType) {
            return solveMethodAsUsage(new ReferenceTypeImpl(new ReflectionClassDeclaration(Object.class, typeSolver), typeSolver), name, argumentsTypes, typeSolver, invokationContext);
        } else {
            throw new UnsupportedOperationException("type usage: " + type.getClass().getCanonicalName());
        }
    }
    private Type usingParameterTypesFromScope(Type scope, Type type, Map<TypeParameterDeclaration, Type> inferredTypes) {
        if (type.isReferenceType()) {
            for (Tuple2<TypeParameterDeclaration, Type> entry : type.asReferenceType().getTypeParametersMap()) {
                if (entry._1.declaredOnType() && scope.asReferenceType().getGenericParameterByName(entry._1.getName()).isPresent()) {
                    type = type.replaceTypeVariables(entry._1, scope.asReferenceType().getGenericParameterByName(entry._1.getName()).get(), inferredTypes);
                }
            }
            return type;
        } else {
            return type;
        }
    }
    private Type applyInferredTypes(Type type, Map<TypeParameterDeclaration, Type> inferredTypes) {
        for (TypeParameterDeclaration tp : inferredTypes.keySet()) {
            type = type.replaceTypeVariables(tp, inferredTypes.get(tp), inferredTypes);
        }
        return type;
    }
}
