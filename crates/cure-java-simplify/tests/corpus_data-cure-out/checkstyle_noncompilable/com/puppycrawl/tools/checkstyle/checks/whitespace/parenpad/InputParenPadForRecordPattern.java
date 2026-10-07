package com.puppycrawl.tools.checkstyle.checks.whitespace.parenpad;

public class InputParenPadForRecordPattern {
    void test(Object obj) {}
    void test2(Object obj) {
        switch (obj) {
            case ColoredPoint(Point p, String c) -> {}
            case Point( int x, int y) when x == 0 -> {}
            case Point( int x, int y ) -> {}
            default -> {}
        }
        switch (obj) {
            case ColoredPoint(Point(int x, int _), String c) when x == 0 -> {}
            case ColoredPoint( Point( int x, int _), String c) when c.isEmpty() -> {}
            case ColoredPoint(Point( int x, int _ ), String c ) when x == 0 -> {}
            case ColoredPoint( Point( int x, int _ ), String c ) when c.isEmpty() -> {}
            default -> {}
        }
    }
    record ColoredPoint(Point p, String color) {
    }
    record Point(int x, int y) {
    }
}
