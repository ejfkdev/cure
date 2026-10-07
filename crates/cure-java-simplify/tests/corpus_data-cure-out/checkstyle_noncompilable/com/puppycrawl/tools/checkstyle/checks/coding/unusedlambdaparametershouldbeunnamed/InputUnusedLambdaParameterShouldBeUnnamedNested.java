package com.puppycrawl.tools.checkstyle.checks.coding.unusedlambdaparametershouldbeunnamed;

import java.util.function.BiFunction;
import java.util.function.Function;

public class InputUnusedLambdaParameterShouldBeUnnamedNested {
    void m1() {
        BiFunction<String, String, String> function = (x, y) -> {
            Function<String, String> function2 = (z) -> {
                return "a";
            };
            return "a";
        };
        function = (_, _) -> {
            Function<String, String> function2 = (z) -> {
                return "a";
            };
            return "a";
        };
        function = (_, _) -> {
            Function<String, String> function2 = (_) -> {
                return "a";
            };
            return "a";
        };
        function = (x, y) -> {
            Function<String, String> function2 = (z) -> {
                return x;
            };
            return y + 0;
        };
        function = (x, y) -> {
            BiFunction<String, String, String> function2 = (z, w) -> {
                return x + w;
            };
            return "0";
        };
        function = (x, y) -> {
            BiFunction<String, String, String> function2 = (z, w) -> {
                return x + y;
            };
            return "0";
        };
        function = (x, y) -> {
            BiFunction<String, String, String> function2 = (z, w) -> {
                return x + y + z;
            };
            return "a";
        };
    }
    void TypedLambdaParameter() {
        BiFunction<String, String, String> function = (String x, String y) -> {
            Function<Integer, String> function2 = (Integer z) -> {
                return "a" + x;
            };
            return "a";
        };
        function = (String x, String y) -> {
            Function<Integer, String> function2 = (Integer z) -> {
                return "a" + z;
            };
            return y;
        };
        function = (String x, String y) -> {
            Function<Integer, String> function2 = (Integer _) -> {
                return "a" + x + y;
            };
            return x;
        };
    }
}
