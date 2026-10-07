package com.puppycrawl.tools.checkstyle.checks.finalparameters;

public class InputFinalParametersPatternVariables {
    record ARecord(String name, int age) {
    }
    static void method(final Object o) {
        boolean isString = o instanceof String s;
        boolean isStringCorrect = o instanceof String correct;
        switch (o) {
            case String s -> {}
            case ARecord(String name, final int age) -> {}
            default -> {}
        }
        switch (o) {
            case String s -> {}
            case ARecord(final String name, final int age) -> {}
            default -> {}
        }
    }
}
