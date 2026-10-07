package com.puppycrawl.tools.checkstyle.checks.whitespace.operatorwrap;

public class InputOperatorWrapInstanceOfOperatorEndOfLine {
    void test(Object o) {}
    void test2(Object o) {
        boolean e = o instanceof Integer i;
    }
    void test3(Object o) {
        switch (o) {
            case Number n when n instanceof
                   Integer:
                {}
                break;
            default:
                {}
        }
        switch (o) {
            case Number n when n instanceof Integer:
                {}
                break;
            default:
                {}
        }
        switch (o) {
            case Number i when i instanceof
                   Integer _:
                {}
                break;
            default:
                {}
        }
        switch (o) {
            case Number n when n
                  // violation below ''instanceof' should be on the previous line.'
                   instanceof Integer:
                {}
                break;
            default:
                {}
        }
    }
    void test4(Object o) {
        switch (o) {
            case Object obj when obj instanceof
                    Point(int _, int _):
                {}
                break;
            default:
                {}
        }
        switch (o) {
            case Object obj when obj
                   // violation below ''instanceof' should be on the previous line.'
                    instanceof Point(int _, int _):
                {}
                break;
            default:
                {}
        }
    }
    record Point(int x,int y) {
    }
}
