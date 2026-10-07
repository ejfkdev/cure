package com.puppycrawl.tools.checkstyle.checks.naming.illegalidentifiername;

public class InputIllegalIdentifierNameRecordPattern {
    void m(Object o) {}
    void m2(Object o) {
        switch (o) {
            case Point(int permits, int yield):
                {}
                break;
            case ColorPoint(Point(int permit$, int _), String _):
                {}
            default:
                {}
        }
    }
    record Point(int x, int y) {
    }
    record ColorPoint(Point p, String color) {
    }
}
