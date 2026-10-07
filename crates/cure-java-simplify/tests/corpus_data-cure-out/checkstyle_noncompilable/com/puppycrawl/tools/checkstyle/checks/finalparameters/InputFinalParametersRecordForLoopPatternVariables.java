package com.puppycrawl.tools.checkstyle.checks.finalparameters;

public class InputFinalParametersRecordForLoopPatternVariables {
    record ARecord(String name, int age) {
    }
    static void method(final ARecord[] records) {
        for (ARecord(String name, final int age) : records) {
                }
        for (ARecord(final String name, final int age) : records) {
                }
    }
}
