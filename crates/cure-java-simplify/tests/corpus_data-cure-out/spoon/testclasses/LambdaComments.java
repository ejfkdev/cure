package spoon.test.comment.testclasses;

import java.util.function.BiFunction;

public class LambdaComments {
    void m1() {}
    void m2() {
        BiFunction<Integer, Integer, Integer> lambda6 = (a, b) -> {
            return a + b;
        };
        BiFunction<Integer, Integer, Integer> lambda7 = (a, b) -> {
            return a + b;
        };
        BiFunction<Integer, Integer, Integer> lambda8 = (a, b) -> {
            return a + b;
        };
    }
    void m3() {}
}
