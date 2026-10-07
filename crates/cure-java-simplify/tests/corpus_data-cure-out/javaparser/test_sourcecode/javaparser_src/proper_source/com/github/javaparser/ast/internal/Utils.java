package com.github.javaparser.ast.internal;

import java.util.Collection;
import java.util.Collections;
import java.util.List;

public class Utils {
    public static <T> List<T> ensureNotNull(List<T> list) {
        return list == null ? Collections.emptyList() : list;
    }
    public static <E> boolean isNullOrEmpty(Collection<E> collection) {
        return collection == null || collection.isEmpty();
    }
}
