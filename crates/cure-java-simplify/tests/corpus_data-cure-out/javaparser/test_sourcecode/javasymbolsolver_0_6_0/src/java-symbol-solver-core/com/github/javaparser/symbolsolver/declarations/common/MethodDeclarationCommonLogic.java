package com.github.javaparser.symbolsolver.declarations.common;

import com.github.javaparser.symbolsolver.core.resolution.Context;
import com.github.javaparser.symbolsolver.logic.InferenceContext;
import com.github.javaparser.symbolsolver.model.declarations.MethodDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.TypeParameterDeclaration;
import com.github.javaparser.symbolsolver.model.methods.MethodUsage;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import com.github.javaparser.symbolsolver.model.typesystem.TypeVariable;
import com.github.javaparser.symbolsolver.reflectionmodel.MyObjectProvider;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public class MethodDeclarationCommonLogic {
    private MethodDeclaration methodDeclaration;
    private TypeSolver typeSolver;
    public MethodDeclarationCommonLogic(MethodDeclaration methodDeclaration, TypeSolver typeSolver) {
        this.methodDeclaration = methodDeclaration;
        this.typeSolver = typeSolver;
    }
    public MethodUsage resolveTypeVariables(Context context, List<Type> parameterTypes) {
        Type returnType = replaceTypeParams(methodDeclaration.getReturnType(), typeSolver, context);
        List<Type> params = new ArrayList<>();
        for (int i = 0; i < methodDeclaration.getNumberOfParams(); i++) {
            Type replaced = replaceTypeParams(methodDeclaration.getParam(i).getType(), typeSolver, context);
            params.add(replaced);
        }
        InferenceContext inferenceContext = new InferenceContext(MyObjectProvider.INSTANCE);
        for (int i = 0; i < methodDeclaration.getNumberOfParams() - (methodDeclaration.hasVariadicParameter() ? 1 : 0); i++) {
            Type formalParamType = methodDeclaration.getParam(i).getType();
            Type actualParamType = parameterTypes.get(i);
            inferenceContext.addPair(formalParamType, actualParamType);
        }
        returnType = inferenceContext.resolve(inferenceContext.addSingle(returnType));
        return new MethodUsage(methodDeclaration, params, returnType);
    }
    private Type replaceTypeParams(Type type, TypeSolver typeSolver, Context context) {
        if (type.isTypeVariable()) {
            TypeParameterDeclaration typeParameter = type.asTypeParameter();
            if (typeParameter.declaredOnType()) {
                Optional<Type> typeParam = typeParamByName(typeParameter.getName(), typeSolver, context);
                if (typeParam.isPresent()) {
                    type = typeParam.get();
                }
            }
        }
        if (type.isReferenceType()) {
            type.asReferenceType().transformTypeParameters((tp) -> replaceTypeParams(tp, typeSolver, context));
        }
        return type;
    }
    protected Optional<Type> typeParamByName(String name, TypeSolver typeSolver, Context context) {
        return methodDeclaration.getTypeParameters().stream().filter((tp) -> tp.getName().equals(name)).map((tp) -> toType(tp)).findFirst();
    }
    protected Type toType(TypeParameterDeclaration typeParameterDeclaration) {
        return new TypeVariable(typeParameterDeclaration);
    }
}
