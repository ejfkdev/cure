package com.github.javaparser.ast.nodeTypes;

import com.github.javaparser.ast.type.Type;
import java.util.*;
import static com.github.javaparser.utils.Utils.arrayToList;

public interface NodeWithTypeArguments<T> {
    List<Type<?>> getTypeArguments();
    T setTypeArguments(List<Type<?>> typeArguments);
    default boolean isUsingDiamondOperator() {
        return getTypeArguments() == null ? false : getTypeArguments().isEmpty();
    }
    default T setDiamondOperator() {
        setTypeArguments(new LinkedList<>());
        return (T) this;
    }
    default T removeTypeArguments() {
        setTypeArguments((List<Type<?>>) null);
        return (T) this;
    }
    default T setTypeArguments(Type<?>... typeArguments) {
        setTypeArguments(arrayToList(typeArguments));
        return (T) this;
    }
}
