package com.github.javaparser.symbolsolver.reflectionmodel.comparators;

import java.util.Comparator;

public class ClassComparator implements Comparator<Class<?>> {
    @Override
    public int compare(Class<?> o1, Class<?> o2) {
        int subCompare = o1.getCanonicalName().compareTo(o2.getCanonicalName());
        if (subCompare != 0) 
            return subCompare;
        subCompare = Boolean.compare(o1.isAnnotation(), o2.isAnnotation());
        if (subCompare != 0) 
            return subCompare;
        subCompare = Boolean.compare(o1.isArray(), o2.isArray());
        if (subCompare != 0) 
            return subCompare;
        subCompare = Boolean.compare(o1.isEnum(), o2.isEnum());
        if (subCompare != 0) 
            return subCompare;
        subCompare = Boolean.compare(o1.isInterface(), o2.isInterface());
        return subCompare != 0 ? subCompare : 0;
    }
}
