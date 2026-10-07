package com.github.javaparser.ast.nodeTypes;

import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.ClassOrInterfaceType;

public interface NodeWithImplements<T> {
    public List<ClassOrInterfaceType> getImplements();
    public T setImplements(List<ClassOrInterfaceType> implementsList);
    @SuppressWarnings("unchecked")
    public default T addImplements(String name) {
        ClassOrInterfaceType classOrInterfaceType = new ClassOrInterfaceType(name);
        getImplements().add(classOrInterfaceType);
        classOrInterfaceType.setParentNode((Node) this);
        return (T) this;
    }
    public default T addImplements(Class<?> clazz) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addImplements(clazz.getSimpleName());
    }
}
