package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

public class InputAbbreviationAsWordInNameCheckRecordPatterns {
    void m(Object o) {}
    void m2(Object o) {
        switch (o) {
            case ColoredPoint(int x, int y, String COLOR) -> {}
            case Rectangle(ColoredPoint(int x, int INTEGER, String COLOR), _) -> {}
            default -> {}
        }
    }
    record ColoredPoint(int x, int y, String color) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
}
