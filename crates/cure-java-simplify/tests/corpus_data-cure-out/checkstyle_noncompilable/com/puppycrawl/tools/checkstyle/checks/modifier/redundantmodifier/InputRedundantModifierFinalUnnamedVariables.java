package com.puppycrawl.tools.checkstyle.checks.modifier.redundantmodifier;

import java.util.function.BiFunction;

public class InputRedundantModifierFinalUnnamedVariables {
    void m(Object o) {
        if (o instanceof String s) 
            if (o instanceof String _s) {}
        int _ = sideEffect();
        int __ = sideEffect();
        int x = sideEffect();
        int _x = sideEffect();
    }
    void m2(Object o) {
        switch (o) {
            case String _ -> {}
            case Float _ -> {}
            case Integer __ -> {}
            case Double s -> {}
            default -> {}
        }
    }
    void m3() {
        try (var a = lock()) {} catch (Exception e) {}
        try (var _ = lock()) {} catch (Exception _) {}
    }
    void m4() {
        BiFunction<Integer, Integer, Integer> f = (final Integer a, final Integer b) -> {
            return 5;
        };
        BiFunction<Integer, Integer, Integer> f2 = (final Integer _, final Integer b) -> {
            return 5;
        };
        BiFunction<Integer, Integer, Integer> f3 = (final Integer _, final Integer _) -> {
            return 5;
        };
    }
    int sideEffect() {
        return 0;
    }
    AutoCloseable lock() {
        return null;
    }
    record Point(int x, int y) {
    }
}
