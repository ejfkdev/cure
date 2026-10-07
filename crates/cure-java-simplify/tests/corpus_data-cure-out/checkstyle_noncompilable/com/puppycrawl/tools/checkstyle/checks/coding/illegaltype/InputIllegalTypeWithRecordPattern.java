package com.puppycrawl.tools.checkstyle.checks.coding.illegaltype;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;

public class InputIllegalTypeWithRecordPattern {
    record ColoredPoint(Point p, String c) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
    record Point(int x, int y) {
    }
    record Box<T>(T t) {
    }
    public void testWithInstanceOf(Object obj) {}
    public void testWithInstanceOfWithGenerics() {
        Box<LinkedHashMap<Integer,Integer>> box = new Box<>(new LinkedHashMap<>());
    }
    public void testWithSwitch(Object obj) {
        switch (obj) {
            case Point(_,_) -> System.out.println("point");
            case Rectangle(ColoredPoint(Point(_, _),_),_) -> System.out.println("rectangle");
            case ColoredPoint _ -> System.out.println("coloredPoint");
            default -> throw new IllegalStateException("Unexpected value: " + obj);
        }
    }
}
