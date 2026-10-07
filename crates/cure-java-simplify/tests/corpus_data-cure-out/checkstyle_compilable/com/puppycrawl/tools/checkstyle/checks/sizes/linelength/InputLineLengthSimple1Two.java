package com.puppycrawl.tools.checkstyle.checks.sizes.linelength;

final public class InputLineLengthSimple1Two {
    private void longMethod() {}
    private InputLineLengthSimple1Two() {}
    private void localVariables() {
        for (int k = 0; k < 1; k++) {}
        for (int I = 0; I < 1; I++) {}
    }
    void ALL_UPPERCASE_METHOD() {}
    private static final int BAD__NAME = 3;
}
