package com.puppycrawl.tools.checkstyle.checks.suppresswarningsholder;

public class InputSuppressWarningsHolder2 {
    static final String unusedLocalVariableCheck = "UnusedLocalVariableCheck";
    static final String localVariableNameCheck = "LocalVariableNameCheck";
    void test2() {
        @SuppressWarnings({InputSuppressWarningsHolder2.unusedLocalVariableCheck,
                        InputSuppressWarningsHolder2.localVariableNameCheck})
                int a;
    }
    void test3() {
        @SuppressWarnings({unusedLocalVariableCheck, localVariableNameCheck})
                int a;
    }
    void test1() {
        @SuppressWarnings(InputSuppressWarningsHolder2.localVariableNameCheck)
                int a;// violation 'Unused local variable 'a''
    }
    void test4() {
        @SuppressWarnings(localVariableNameCheck)
                int a;// violation 'Unused local variable 'a''
    }
}
