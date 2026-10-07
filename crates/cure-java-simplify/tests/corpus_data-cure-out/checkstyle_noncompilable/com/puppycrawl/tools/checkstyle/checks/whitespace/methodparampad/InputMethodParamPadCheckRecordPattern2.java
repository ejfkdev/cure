package com.puppycrawl.tools.checkstyle.checks.whitespace.methodparampad;

public class InputMethodParamPadCheckRecordPattern2 {
    void test(Object obj) {
        switch (obj) {
            case ColoredPoint (Point  ( int x, int y), String s) -> {}
            case ColoredPoint (Point p, String s) -> {}
            case Point(int x, int y) -> {}
            default -> {}
        }
    }
    record ColoredPoint(Point p, String s) {
    }
    record Point(int x, int y) {
    }
}
