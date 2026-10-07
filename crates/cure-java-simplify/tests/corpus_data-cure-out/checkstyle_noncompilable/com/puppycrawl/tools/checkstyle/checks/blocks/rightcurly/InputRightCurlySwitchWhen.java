package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

record ColoredPoint(int p, int x, String c) {
}

record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
}

public class InputRightCurlySwitchWhen {
    public void testSwitchRuleWhenGuard() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x >= 2) -> {}
            case ColoredPoint(int x, _, _) when (x == 1) -> {}
            case ColoredPoint(int x, _, _) when (x == 0) -> {}
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard2() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x >= 5) -> {}
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard3() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 7) -> {}
            case ColoredPoint(int x, _, _) when (x == 8) -> {}
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard4() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 9) -> {}
            case ColoredPoint(int x, _, _) when (x == 10) -> {}
            default -> {}
        }
    }
    public void testSwitchRuleWhenGuard5() {
        Object obj = new Object();
        switch (obj) {
            case ColoredPoint(int x, _, _) when (x == 11) -> {}
            case ColoredPoint(int x, _, _) when (x == 12) -> {}
            default -> {}
        }
    }
}
