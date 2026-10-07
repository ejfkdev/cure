package com.github.javaparser.ast.nodeTypes;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.Type;

public interface NodeWithType<T> {
    Type<?> getType();
    T setType(Type<?> type);
    default T setType(Class<?> typeClass) {
        ((Node) this).tryAddImportToParentCompilationUnit(typeClass);
        return setType(new ClassOrInterfaceType(typeClass.getSimpleName()));
    }
    default T setType(final String type) {
        return setType(new ClassOrInterfaceType(type));
    }
}
