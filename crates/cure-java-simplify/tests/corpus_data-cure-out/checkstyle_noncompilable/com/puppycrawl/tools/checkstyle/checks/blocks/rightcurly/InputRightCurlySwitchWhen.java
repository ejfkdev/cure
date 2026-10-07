package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

record ColoredPoint(int p, int x, String c) {
}

record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
}

public class InputRightCurlySwitchWhen {
    public void testSwitchRuleWhenGuard() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x >= 2) -> {
                int y = 0;
            }
            case ColoredPoint(int x, _, _) when (x == 1) -> {
                int y = 1;
            }
            case ColoredPoint(int x, _, _) when (x == 0) -> {
                int y = 2;
            }
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard2() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x >= 5) -> {
                int y = 3;
            }
            default -> {
                int z = 4;
            }
        }
    }
    public void testSwitchRuleWhenGuard3() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 7) -> {
                int a = 1;
            }
            case ColoredPoint(int x, _, _) when (x == 8) -> {
                int b = 2;
            }
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard4() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 9) -> {
                int x1 = 10;
            }
            case ColoredPoint(int x, _, _) when (x == 10) -> {
                int x2 = 20;
            }
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard5() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 11) -> {
                int v = 1;
            }
            case ColoredPoint(int x, _, _) when (x == 12) -> {
                int v = 2;
            }
            default -> {}
        }
    }
}
