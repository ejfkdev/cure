package com.puppycrawl.tools.checkstyle.checks.coding.modifiedcontrolvariable;

import java.util.List;

public class InputModifiedControlVariableRecordDecomposition {
    static void m() {
        List<Point> points = List.of(new Point(1, 2), new Point(3, 4));
        for (Point(var x, var y): points) {}
        for (Point(Integer x, Integer y): points) {}
        for (Point p : points) {
            var x = p.x();
            var y = p.y();
        }
        for (Point p : points) {
            var x = p.x();
            var y = p.y();
            p = new Point(1, 2);
        }
        for (Point(var x, var y): points) {
                    x = 1;
                    y = 2;
                }
    }
    public static void main(String[] args) {
        m();
    }
    record Point(Integer x, Integer y) {
    }
}
