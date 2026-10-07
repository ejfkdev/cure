package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.Callable;

public class InputAntlr4AstRegressionWeirdCtor {
    void meth() {
        List<Integer> x = new ArrayList<Integer>();
        List<List<Integer>> y = new ArrayList<List<Integer>>();
        List< Integer > a = new ArrayList< Integer >();
        List< List < Integer > > b = new ArrayList< List < Integer > >();
    }
    public int compareTo(Object aObject) {
        return 0;
    }
    public static <T> Callable<T> callable(Runnable task, T result) {
        return null;
    }
    public static <T> Callable<T> callable2(Runnable task, T result) {
        Map<Class<?>, Integer> x = new HashMap<Class<?>, Integer>();
        for (Map.Entry<Class<?>, Integer> entry : x.entrySet()) {
            entry.getValue();
        }
        Class<?>[] parameterClasses = new Class<?>[0];
        return null;
    }
    public int getConstructor(Class<?>... parameterTypes) {
        Collections.emptySet();
        Collections.emptySet();
        return 666;
    }
    <T> InputAntlr4AstRegressionWeirdCtor(List<T> things, int i) {}
    public <T> InputAntlr4AstRegressionWeirdCtor(List<T> things) {}
    public interface IntEnum {
    }
    public static class IntEnumValueType<E extends Enum<E> & IntEnum> {
    }
    public static class IntEnumValueType2<E extends Enum<E>& IntEnum> {
    }
    public static class IntEnumValueType3<E extends Enum<E>  & IntEnum> {
    }
    public static class IntEnumValueType4<T extends Comparable<List<T>> & IntEnum> {
    }
    public void beforeAndAfter() {
        List<
Integer> x = new ArrayList<Integer
>();
        List<Integer> y = new ArrayList<Integer
        >();
        Map<Class<?>, Integer> b = (Map<Class<?>, Integer>) new HashMap<Class<?>, Integer>();
    }
    Object ok;
    ();
    Object notOkStart;
    (); // violation
    Object notOkEnd;
    (); // violation
    Object notOkStartAndEnd;
    (); // violation
    Object okWithPackage;
    ();
    Object ok2;
    ();
    Object notOkStart2;
    (); // violation
    Object notOkEnd2;
    (); // violation
    Object notOkStartAndEnd2;
    (); // violation
}

interface SupplierFunction<T> extends Map<List<T>, T> {
}

class Outer {
    static class Inner {
    }
}
