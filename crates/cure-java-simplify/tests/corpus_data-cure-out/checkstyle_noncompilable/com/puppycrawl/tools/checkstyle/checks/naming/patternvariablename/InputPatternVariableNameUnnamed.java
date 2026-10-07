package com.puppycrawl.tools.checkstyle.checks.naming.patternvariablename;

public class InputPatternVariableNameUnnamed {
    void test(Object o, Object obj) {
        if (o instanceof String __ && __.length() == 0) {}
    }
    void test2(Object o) {
        switch (o) {
            case String __ when __.length() == 0 -> {}
            case Integer _ -> {}
            case Double _s -> {}
            default -> {}
        }
    }
    void test3(Object o, Object obj) {
        switch (o) {
            case Point(int _, int _) -> {}
            case ColoredPoint(Point(int _, int x), String _Color) -> {}
            default -> {}
        }
        boolean b = o instanceof Point(int _, int y) && obj instanceof ColoredPoint(Point(int _, int x), String __);
    }
    record Point(int x, int y) {
    }
    record ColoredPoint(Point point, String color) {
    }
}
