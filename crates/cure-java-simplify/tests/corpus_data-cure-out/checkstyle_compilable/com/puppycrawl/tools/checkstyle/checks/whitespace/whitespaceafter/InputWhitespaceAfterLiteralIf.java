package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespaceafter;

public class InputWhitespaceAfterLiteralIf {
    boolean condition() {
        return false;
    }
    void testIfElse() {
        if (condition()) {
            testIfElse();
        } else {
            testIfElse();
        }
        if (condition()) {
            testIfElse();
        } else {
            testIfElse();
        }
        if (condition()) {
            testIfElse();
        } else {
            testIfElse();
        }
    }
    void testWhile() {
        while (condition()) {
            testWhile();
        }
        while (condition()) {
            testWhile();
        }
    }
    void testFor() {
        for (int i = 0; i < 5; i++) {
            testFor();
        }
        for (int i = 0; i < 5; i++) {
            testFor();
        }
    }
    void testDo() {
        do {
            testDo();
        } while (condition());
        do {
            testDo();
        } while (condition());
    }
}
