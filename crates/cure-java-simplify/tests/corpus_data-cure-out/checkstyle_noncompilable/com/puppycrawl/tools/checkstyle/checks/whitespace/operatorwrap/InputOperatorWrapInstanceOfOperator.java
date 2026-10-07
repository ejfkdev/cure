package com.puppycrawl.tools.checkstyle.checks.whitespace.operatorwrap;

public class InputOperatorWrapInstanceOfOperator {
    void test(Object o) {}
    void test2(Object o) {}
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
