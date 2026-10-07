package com.puppycrawl.tools.checkstyle.checks.coding.unusedlambdaparametershouldbeunnamed;

import java.util.function.BiFunction;

public class InputUnusedLambdaParameterShouldBeUnnamedMultipleParameters {
    String x, y;
    void testUnused() {
        BiFunction<String, String, String> function = (x, y) -> {
            return "xy";
        };
        function = (x, y) -> {
            return this.x + y;
        };
        function = (x, y) -> this.x + this.y;
        function = (X, Y) -> {
            return new X().s() + Y;
        };
        function = (x, y) -> {
            return "xy";
        };
        function = (X, Y) -> {
            return new X().toString() + Y.toString();
        };
        function = (X, Y) -> {
            x = X;
            return x + Y.toString();
        };
    }
    void testAllUsed() {
        BiFunction<String, String, String> function = (x, y) -> {
            return x + y;
        };
        function = (x, y) -> {
            return this.x + y + x + this.y;
        };
    }
    void testWithUnnamed() {
        BiFunction<String, String, String> function = (_, _) -> {
            return "xy";
        };
        function = (_, y) -> {
            return this.x + this.y + y;
        };
        function = (_, y) -> {
            return this.x + this.y;
        };
        function = (_, _) -> {
            return this.x + this.y;
        };
        function = (_, _) -> {
            return new X().s() + "x";
        };
    }
    class X {
        String s() {
            return "x";
        }
    }
    String x(String x) {
        return x;
    }
    String y(String y) {
        return y;
    }
}
