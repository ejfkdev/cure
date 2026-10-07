package com.puppycrawl.tools.checkstyle.checks.whitespace.nowhitespaceafter;

public class InputNoWhitespaceAfterUnnamedPattern {
    public void test(Object o) {
        switch (o) {
            case Point(int x, _ , String color) -> {}
            default -> {}
        }
    }
    record Point(int x, int y   , String color) {
    }
}
