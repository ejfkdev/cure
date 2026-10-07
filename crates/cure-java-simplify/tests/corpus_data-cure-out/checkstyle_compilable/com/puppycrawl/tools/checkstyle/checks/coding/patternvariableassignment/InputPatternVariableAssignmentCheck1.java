package com.puppycrawl.tools.checkstyle.checks.coding.patternvariableassignment;

public class InputPatternVariableAssignmentCheck1 {
    boolean theMatch;
    private static final Object o = "";
    private static final boolean B1s = o instanceof String s;
    record Rectangle(Object test1, Object test2) {
    }
    record ColoredPoint(Object test1, Object test2, Object test3) {
    }
    public void testAssignment(Object obj) {
        if (obj instanceof String) {
            System.out.println(obj);
        }
        if (obj instanceof String s) {
            s = "hello";
            System.out.println(s);
        }
        if (obj instanceof Rectangle(ColoredPoint x, ColoredPoint y)) {
            x = new ColoredPoint(1, 2, "red");
            y = new ColoredPoint(3, 4, "blue");
        }
        if (obj instanceof Rectangle(ColoredPoint(Integer x1,Integer x2,String c), Integer x)) {
            c = "red";
        }
        if (obj instanceof Rectangle(ColoredPoint(Integer x1, ColoredPoint(Integer y1,Integer y2,
                                                              String d), String c), Integer x)) {
            c = "red";
        }
        if (obj instanceof Rectangle(ColoredPoint(Integer x1,Integer x2,String c),
                                     ColoredPoint(Integer y1,Integer y2, String d))) {
            c = "red";
        }
        if (obj instanceof Integer d) {
            for (int i = 0; i < 1; i++) {
                if (d > 5) {
                    d -= 3;
                }
            }
        }
        if (obj instanceof Integer t) ;
        Rectangle antiFigure = obj instanceof Rectangle f ? (f = null) : new Rectangle(40, 40);
        if (obj instanceof String rectName) {
            this.theMatch = testBooleans(obj);
        }
        record ColoredRectangle() {};
        if (obj instanceof String[] sa) {
            for (int i = 0; i < sa.length; i++) {
                if (sa[i] == null) {
                    sa[i] = sa[i - 1];
                }
            }
        }
        if (!(obj instanceof Integer)) {
            assert obj instanceof Double s;
        }
    }
    public boolean testBooleans(Object obj) {
        return obj instanceof Boolean bool ? bool : obj instanceof String s;
    }
}
