package com.puppycrawl.tools.checkstyle.grammar;

import java.lang.reflect.ParameterizedType;
import java.lang.reflect.WildcardType;
import java.util.List;

public class InputMultiDimensionalArraysInGenerics {
    @SuppressWarnings("unused") void withUpperBound(List<? extends int[][]> list) {}
    @SuppressWarnings("unused") void withLowerBound(List<? super String[][]> list) {}
    @SuppressWarnings("unused") void withLowerBound2(List<? super String[][][]> list) {}
    static WildcardType getWildcardType(String methodName) throws Exception {
        return (WildcardType) ((ParameterizedType) WildcardType.class.getDeclaredMethod(methodName, List.class).getGenericParameterTypes()[0]).getActualTypeArguments()[0];
    }
}
