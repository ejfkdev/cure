package com.puppycrawl.tools.checkstyle.grammar;

public class InputRecordPatternsNestedDecomposition {
    record A(Object o) {
    }
    record B(Object o) {
    }
    record Point(int x, int y) {
    }
    enum Color {
        RED, GREEN, BLUE
    }
    record ColoredPoint(Point p, Color c) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
    void method(Object param) {
        switch (param) {
            case A(Object o) -> {}
            case B(var o) -> {}
            default -> {}
        }
    }
    static void p1(Rectangle r) {
        if (r instanceof Rectangle(ColoredPoint ul,ColoredPoint lr)) {
            System.out.println(ul.c());
        }
    }
    static void p2(Rectangle r) {
        if (r instanceof Rectangle(ColoredPoint(Point p1,Color c1),
                ColoredPoint lr1) && r instanceof Rectangle(ColoredPoint(Point p2,Color c2),
                ColoredPoint lr2) && lr2.c == Color.BLUE) {
            System.out.println(r);
        }
    }
    static void p3(Rectangle r) {
        if (r instanceof Rectangle(ColoredPoint(Point p1,Color c1),
                ColoredPoint lr1) && r instanceof Rectangle(
                ColoredPoint(Point(int x,int y),Color c2),
                ColoredPoint lr2)) {
            System.out.println(r);
        }
    }
    static void p4() {
        Color c1 = Color.BLUE;
        Color c2 = Color.GREEN;
        Rectangle r = new Rectangle(new ColoredPoint(new Point(0, 0), c1), new ColoredPoint(new Point(0, 0), c2));
        if (r instanceof Rectangle(
                ColoredPoint(Point(var x,var y),var c),
                var lr)) {
            System.out.println(x);
        }
    }
}
