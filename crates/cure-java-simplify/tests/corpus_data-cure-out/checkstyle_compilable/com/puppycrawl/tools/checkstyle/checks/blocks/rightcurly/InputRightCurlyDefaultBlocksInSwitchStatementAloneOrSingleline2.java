package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

public class InputRightCurlyDefaultBlocksInSwitchStatementAloneOrSingleline2 {
    public static void test10() {
        switch (0) {
            case 0:
                int x = 1;
            case 1:
                x = 1;
                break;
            default:
                {
                    x = 1;
                }
            case 2:
                x = 5;
        }
    }
    public static void test11() {
        switch (0) {
            case 0:
                {
                    int x = 0;
                }
            default:
                {}
            case 1:
                int x = 1;
                break;
        }
    }
    public static void test12() {
        switch (0) {
            case 0:
                {}
            default:
                {
                    int y;
                }
            case 1:
                int x = 1;
                break;
        }
    }
    public static void test13() {
        switch (0) {
            case 0:
                {}
            default:
                {}
            case 1:
                break;
        }
    }
    public static void test14() {
        switch (0) {
            case 0:
                {
                    int x = 1;
                }
            default:
                {}
            case 1:
                {
                    break;
                }
        }
    }
    public static void test15() {
        switch (0) {
            default:
                int x = 1;
                {}
                break;
            case 1:
                {}
                int y = 1;
                break;
        }
    }
    public static void test16() {
        int mode = 0;
        switch (mode) {
            case 0:
                int x = 1;
                {}
            default:
                mode++;
                {}
                int y;
            case 3:
                {}
                int z = 1;
        }
    }
    public static void test17() {
        switch (0) {
            default:
                {}
            case 1:
                int z;
                {}
                break;
            case 2:
                break;
        }
    }
    public static void test18() {
        switch (0) {
            case 1:
            default:
                {
                    int x = 0;
                }
                break;
        }
    }
}
