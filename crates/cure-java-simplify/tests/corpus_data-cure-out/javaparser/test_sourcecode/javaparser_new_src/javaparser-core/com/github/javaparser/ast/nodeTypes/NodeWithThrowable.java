package com.github.javaparser.ast.nodeTypes;

import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.ReferenceType;

public interface NodeWithThrowable<T> {
    T setThrows(List<ReferenceType> throws_);
    List<ReferenceType> getThrows();
    @SuppressWarnings("unchecked")
    default T addThrows(ReferenceType throwType) {
        getThrows().add(throwType);
        throwType.setParentNode((Node) this);
        return (T) this;
    }
    default T addThrows(Class<? extends Throwable> clazz) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addThrows(new ClassOrInterfaceType(clazz.getSimpleName()));
    }
    public default boolean isThrows(Class<? extends Throwable> clazz) {
        return isThrows(clazz.getSimpleName());
    }
    public default boolean isThrows(String throwableName) {
        return getThrows().stream().anyMatch((t) -> t.toString().equals(throwableName));
    }
}
