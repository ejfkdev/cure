package com.puppycrawl.tools.checkstyle.checks.suppresswarningsholder;

public class InputSuppressWarningsHolder1 {
    static final String unusedLocalVariableCheck = "UnusedLocalVariableCheck";
    void test() {
        @SuppressWarnings(unusedLocalVariableCheck)
                int a;
    }
    void test2() {
        @SuppressWarnings(InputSuppressWarningsHolder1.unusedLocalVariableCheck)
                int a;
    }
}
