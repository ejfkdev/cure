package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationPatternMatchingForSwitch {
    record Point(int x, int y) {
    }
    record square(Point upperLeft, Point lowerRight) {
    }
    public void test1() {
        Point p = new Point(1, 2);
        switch (p) {
            case Point(int x, int y)                                        //indent:12 exp:12
            when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                        //indent:12 exp:12
                when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
    }
    public void test2() {
        Point p = new Point(1, 2);
        switch (p) {
            case Point(int x, int y)                                     //indent:20 exp:20
                when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                     //indent:20 exp:20
                    when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                     //indent:20 exp:20
                            when x > 0 && y > 0 -> System.out.println(x);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                     //indent:20 exp:20
            when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
    }
    public void test3() {
        Point p = new Point(1, 2);
        switch (p) {
            case Point(int x, int y)                                         //indent:16 exp:16
                when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                             //indent:12 exp:16 warn
            when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                                     //indent:4 exp:16 warn
    when x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                         //indent:16 exp:16
                            when x > 0 && y > 0 -> System.out.println(x);
            case Point(int x, int y) -> System.out.println("error");
        }
        switch (p) {
            case Point(int x, int y)                                                         //indent:0 exp:16 warn
when                                                                        //indent:0 exp:16 warn
x > 0 && y > 0 -> System.out.println(x + y);
            case Point(int x, int y) -> System.out.println("error");
        }
    }
}
