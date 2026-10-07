package com.puppycrawl.tools.checkstyle.checks.naming.parametername;

public final class InputParameterNameMisc {
    private void localVariables() {
        for (int k = 0; k < 1; k++) {
            String innerBlockVariable = "";
        }
        for (int I = 0; I < 1; I++) {
            String InnerBlockVariable = "";
        }
    }
    void ALL_UPPERCASE_METHOD() {}
    private static final int BAD__NAME = 3;
    void errorColumnAfterTabs() {
        int tab5 = 1;
    }
    void veryLong() {}
    void toManyArgs(int aArg1, int aArg2, int aArg3, int aArg4, int aArg5, int aArg6, int aArg7, int aArg8, int aArg9) {}
}
