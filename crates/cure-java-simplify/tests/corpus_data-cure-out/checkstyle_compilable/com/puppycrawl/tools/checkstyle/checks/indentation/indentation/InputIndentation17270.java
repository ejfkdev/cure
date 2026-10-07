package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentation17270 {
    private void test() {}
    private void testWrappedCaseLabels() {
        System.out.println(switch (0) {
            case 0 -> "a";
            case 1, 2, 3, 4 -> "multiple";
            default -> "d";
        });
    }
}
