package spoon.test.comment.testclasses;

import java.util.function.BiFunction;

public class LambdaComments {
    void m1() {
        BiFunction<Integer, Integer, Integer> lambda5 = (a, b) -> a + b;
    }
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
    void m3() {
        BiFunction<Integer, Integer, Integer> lambda13 = (/* param1 */ a /* param1 */, /* param2 */ b /* param2 */) -> a + b;
    }
}
