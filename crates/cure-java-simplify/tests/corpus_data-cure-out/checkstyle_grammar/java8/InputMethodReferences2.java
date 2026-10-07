package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.util.function.Function;
import java.util.function.Supplier;

public class InputMethodReferences2 {
    public static void main(String[] args) {
        Function<Integer, String[]> messageArrayFactory = String[]::new;
    }
    private class Bar<T> {
    }
}
