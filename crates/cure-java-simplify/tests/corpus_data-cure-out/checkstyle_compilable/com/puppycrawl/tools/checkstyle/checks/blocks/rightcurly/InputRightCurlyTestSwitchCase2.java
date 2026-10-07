package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

public class InputRightCurlyTestSwitchCase2 {
    public static void method0() {
        switch (0) {
            case 1:
                int x = 1;
                break;
            default:
                x = 0;
        }
    }
    public static void method1() {
        switch (0) {
            default:
                int x = 0;
        }
    }
    public static void method2() {
        switch (0) {
            case 1:
                int x = 1;
                break;
            default:
                x = 0;
        }
    }
    public static void method3() {
        switch (0) {
            default:
                int x = 0;
        }
    }
    public static void method4() {
        switch (0) {
            default:
                int x = 0;
        }
    }
    public static void method5() {
        switch (0) {
            default:
                int x = 0;
        }
    }
    public static void method6() {
        switch (0) {
            case 0:
                int x = 1;
                break;
            default:
                x = 5;
        }
    }
    public static void method7() {
        switch (0) {
            case 0:
                int x = 1;
                break;
            default:
                x = 5;
        }
    }
    public static void method8() {
        switch (0) {
            case 0:
                int x = 1;
                break;
            case 80:
                x = 1;
                break;
        }
    }
    public static void method9() {
        switch (0) {
            case 0:
                int x = 1;
                break;
            default:
                x = 5;
        }
    }
    public static void method10() {
        switch (0) {
            case 0:
                int x = 1;
                break;
            case 80:
                x = 1;
                break;
        }
    }
    public static void method11() {
        int x = 0;
        switch (0) {
            case 0:
                break;
            case 80:
                x = 1;
                break;
        }
    }
    public static void method12() {
        try {
            switch (5) {
                case 1:
                    try {
                        System.out.println("Number is 1");
                        break;
                    } catch (Exception ignored) {}
            }
        } catch (IllegalArgumentException e) {
            System.err.println(e.getMessage());
        }
    }
}
