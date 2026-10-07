package com.github.javaparser.ast.nodeTypes;

import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.VariableDeclaratorId;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.Type;

public interface NodeWithParameters<T> {
    List<Parameter> getParameters();
    T setParameters(List<Parameter> parameters);
    default T addParameter(Type type, String name) {
        return addParameter(new Parameter(type, new VariableDeclaratorId(name)));
    }
    default T addParameter(Class<?> paramClass, String name) {
        ((Node) this).tryAddImportToParentCompilationUnit(paramClass);
        return addParameter(new ClassOrInterfaceType(paramClass.getSimpleName()), name);
    }
    default T addParameter(String className, String name) {
        return addParameter(new ClassOrInterfaceType(className), name);
    }
    @SuppressWarnings("unchecked")
    default T addParameter(Parameter parameter) {
        getParameters().add(parameter);
        parameter.setParentNode((Node) this);
        return (T) this;
    }
    default Parameter addAndGetParameter(Type type, String name) {
        return addAndGetParameter(new Parameter(type, new VariableDeclaratorId(name)));
    }
    default Parameter addAndGetParameter(Class<?> paramClass, String name) {
        ((Node) this).tryAddImportToParentCompilationUnit(paramClass);
        return addAndGetParameter(new ClassOrInterfaceType(paramClass.getSimpleName()), name);
    }
    default Parameter addAndGetParameter(String className, String name) {
        return addAndGetParameter(new ClassOrInterfaceType(className), name);
    }
    default Parameter addAndGetParameter(Parameter parameter) {
        getParameters().add(parameter);
        parameter.setParentNode((Node) this);
        return parameter;
    }
    default Parameter getParamByName(String name) {
        return getParameters().stream().filter((p) -> p.getName().equals(name)).findFirst().orElse(null);
    }
    default Parameter getParamByType(String type) {
        return getParameters().stream().filter((p) -> p.getType().toString().equals(type)).findFirst().orElse(null);
    }
    default Parameter getParamByType(Class<?> type) {
        return getParameters().stream().filter((p) -> p.getType().toString().equals(type.getSimpleName())).findFirst().orElse(null);
    }
}
