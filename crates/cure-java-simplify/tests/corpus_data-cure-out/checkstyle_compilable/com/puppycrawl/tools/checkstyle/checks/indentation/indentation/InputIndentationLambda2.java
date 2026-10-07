package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

import java.util.function.BinaryOperator;
import java.util.function.Consumer;

public class InputIndentationLambda2 {
    public <T> Consumer<Integer> par(Consumer<Integer> f1, Consumer<Integer> f2) {
        return f2;
    }
    private void print(int i) {}
    public Consumer<Integer> returnFunctionOfLambda() {
        return par((x) -> print(x * 1), (x) -> print(x * 2));
    }
    public static <T> BinaryOperator<T> returnLambda() {
        return (t1, t2) -> {
            return t1;
        };
    }
    class TwoParams {
        TwoParams(Consumer<Integer> c1, Consumer<Integer> c2) {}
    }
    public void makeTwoParams() {
        TwoParams t0 = new TwoParams((x) -> print(x * 1), (x) -> print(x * 2));
        TwoParams t1 = new TwoParams((x) -> print(x * 1), (x) -> print(x * 2));
    }
}
