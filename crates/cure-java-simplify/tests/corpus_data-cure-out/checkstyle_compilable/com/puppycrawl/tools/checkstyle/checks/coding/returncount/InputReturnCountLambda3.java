package com.puppycrawl.tools.checkstyle.checks.coding.returncount;

import java.util.Optional;
import java.util.concurrent.Callable;
import java.util.function.Supplier;

public class InputReturnCountLambda3 {
    Runnable fieldWithOneReturnInLambda = () -> {
    return;
};
    Callable<Integer> fieldWithTwoReturnInLambda = () -> {
    return hashCode() == 0 ? 0 : 1;
};
    Optional<Integer> methodWithOneReturnInLambda() {
        return Optional.of(hashCode()).filter((i) -> {
            return i > 0;
        });
    }
    Optional<Integer> methodWithTwoReturnInLambda() {
        return Optional.of(hashCode()).filter((i) -> {
            return i > 0;
        });
    }
    Optional<Object> methodWithThreeReturnInLambda(int number) {
        return Optional.of(number).map((i) -> {
            return i == 42 || i == 7;
        });
    }
    int methodWithTwoReturnWithLambdas(final int number) {
        if (hashCode() > 0) {
            new Thread(() -> {}).start();
            return number;
        } else {
            return Optional.of(hashCode()).orElseGet(() -> {
                return number > 0 ? number : 0;
            });
        }
    }
    Supplier<Supplier<Integer>> methodWithOneReturnPerLambda() {
        return () -> {
            return () -> {
                return 1;
            };
        };
    }
}
