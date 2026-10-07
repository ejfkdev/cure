package com.puppycrawl.tools.checkstyle.checks.annotation.suppresswarnings;

public class InputSuppressWarningsPatternVariables {
    void test(Object o) {
        @SuppressWarnings("")
                int _ = sideEffect();
        @SuppressWarnings("foo")
                int _ = sideEffect();
    }
    int sideEffect() {
        return 0;
    }
    record ColoredPoint(int x, int y, int z) {
    }
}
