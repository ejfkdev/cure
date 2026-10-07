package com.github.javaparser.ast.nodeTypes;

import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.ClassOrInterfaceType;

public interface NodeWithExtends<T> {
    public List<ClassOrInterfaceType> getExtends();
    public T setExtends(final List<ClassOrInterfaceType> extendsList);
    public default T addExtends(Class<?> clazz) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addExtends(clazz.getSimpleName());
    }
    @SuppressWarnings("unchecked")
    public default T addExtends(String name) {
        ClassOrInterfaceType classOrInterfaceType = new ClassOrInterfaceType(name);
        getExtends().add(classOrInterfaceType);
        classOrInterfaceType.setParentNode((Node) this);
        return (T) this;
    }
}
