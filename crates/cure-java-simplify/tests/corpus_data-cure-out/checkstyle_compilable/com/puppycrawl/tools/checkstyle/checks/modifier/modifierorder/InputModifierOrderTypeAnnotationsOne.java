package com.puppycrawl.tools.checkstyle.checks.modifier.modifierorder;

import java.io.File;
import java.io.IOException;
import java.io.Serializable;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.util.Collection;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public class InputModifierOrderTypeAnnotationsOne extends MyClass {
    private @TypeAnnotation String hello = "Hello, World!";
    private @TypeAnnotation final String jdk = "JDK8";
    @TypeAnnotation private String projectName = "Checkstyle";
    private Map.Entry entry;
    private List<@TypeAnnotation String> strings;
    {
        new @TypeAnnotation Object();
    }
    static {
        new @TypeAnnotation Object();
    }
    public void foo1() {
        new @TypeAnnotation Object();
    }
    public void foo2() {
        Object myObject = new Object();
    }
    class MySerializableClass<T> implements @TypeAnnotation Serializable {
    }
    Map<@TypeAnnotation String, @TypeAnnotation List<@TypeAnnotation String>> documents;
    public <E extends @TypeAnnotation Comparator<E> & @TypeAnnotation Comparable> void foo5() {}
    class Folder<F extends @TypeAnnotation File> {
    }
    Collection<? super @TypeAnnotation File> c;
    List<@TypeAnnotation ? extends Comparable<T>> unchangeable;
    void foo6() throws @TypeAnnotation IOException {}
    public void foo7() {
        boolean isNonNull = "string" instanceof String;
    }
    class Nested {
    }
    class T {
    }
    @Override
    public @TypeAnnotation String toString() {
        return "";
    }
    @Override
    @TypeAnnotation public int hashCode() {
        return 1;
    }
    public @TypeAnnotation int foo8() {
        return 1;
    }
    public @TypeAnnotation boolean equals(Object obj) {
        return super.equals(obj);
    }
    @Override void foo10() {
        super.foo10();
    }
}

class MyClass {
    @MethodAnnotation void foo10() {}
    private @MethodAnnotation void foo11() {}
    public @TypeAnnotation MyClass() {}
    @ConstructorAnnotation public MyClass(String name) {}
}

@Target({
    ElementType.FIELD, ElementType.LOCAL_VARIABLE, ElementType.PARAMETER,
    ElementType.TYPE_PARAMETER, ElementType.TYPE_USE}) @interface TypeAnnotation {
}

@interface MethodAnnotation {
}

@interface ConstructorAnnotation {
}
