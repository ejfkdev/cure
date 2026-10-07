package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationRecordPattern {
    record ColoredPoint(boolean p, int x, int c) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
    void test(Object obj) {}
    void testSwitch(Object obj) {
        switch (obj) {
            case Rectangle(                                                 //indent:12 exp:12
        ColoredPoint _,                                                     //indent:8 exp:16 warn
        ColoredPoint _) -> System.out.println("Rectangle");
            default -> {}
        }
        switch (obj) {
            case Rectangle(                                                 //indent:12 exp:12
                ColoredPoint _,                                             //indent:16 exp:16
                ColoredPoint _) -> System.out.println("Rectangle");
            default -> {}
        }
    }
}
