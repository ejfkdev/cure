package com.puppycrawl.tools.checkstyle.checks.whitespace.nowhitespacebeforecasedefaultcolon;

public class InputNoWhitespaceBeforeCaseDefaultColonPatternMatchingForSwitch {
    void test(Object obj) {
        switch (obj) {
            case ColoredPoint(int a, int b, _) when (a >= b):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b > 100):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b != 1000):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b != 100000):
                {}
                break;
            case Rectangle(_,_):
                {}
            default:
                System.out.println("none");
        }
    }
    void test2(Object obj) {
        switch (obj) {
            case ColoredPoint(int a, int b, _) when (a >= b):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b > 100):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b != 1000):
                {}
                break;
            case ColoredPoint(int a, int b, _) when (b != 100000):
                {}
                break;
            case Rectangle(_, _):
                {}
            default:
                System.out.println("none");
        }
    }
    record ColoredPoint(int p, int x, boolean c) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
}
