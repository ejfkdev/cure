package com.puppycrawl.tools.checkstyle.checks.naming.patternvariablename;

public class InputPatternVariableNameRecordPattern {
    void test(Object o) {}
    void test2(Object o) {
        switch (o) {
            case Point(int XX, int __) when XX > 0 -> {}
            case Point(int x, int _) -> {}
            case ColoredPoint(Point(int x, int yy), String color) -> {}
            case ColoredPoint(_, String S) -> {}
            default -> {}
        }
    }
    record Point(int x, int y) {
    }
    record ColoredPoint(Point point, String color) {
    }
}
