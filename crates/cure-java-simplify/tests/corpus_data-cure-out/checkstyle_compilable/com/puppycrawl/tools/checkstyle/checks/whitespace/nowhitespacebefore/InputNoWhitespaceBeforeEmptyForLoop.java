package com.puppycrawl.tools.checkstyle.checks.whitespace.nowhitespacebefore;

public class InputNoWhitespaceBeforeEmptyForLoop {
    public static void f() {
        for (; ; ) {
            break;
        }
        for (int x = 0; ; ) {
            break;
        }
        for (int x = 0; ; ) {
            break;
        }
        for (int x = 0; x < 10; ) {
            break;
        }
        for (int x = 0; x < 10; ) {
            break;
        }
    }
}
