package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableCheckSwitchExpressionsA {
    void foo1() throws Exception {
        Exception e;
        int a = (int) Math.random();
        int b = (int) Math.random();
        switch (a) {
            case 0:
                e = new Exception();
                break;
            case 1:
                if (b == 0) {
                    e = new Exception();
                    break;
                }
                e = new Exception();
                break;
            case 2:
                if (b == 0) {
                    return;
                }
                e = new Exception();
                break;
            default:
                e = new Exception();
                break;
        }
        throw e;
    }
    void foo2() throws Exception {
        Exception e;
        int a = (int) Math.random();
        int b = (int) Math.random();
        switch (a) {
            case 0 -> {
                e = new Exception();
            }
            case 1 -> {
                if (b == 0) {
                    e = new Exception();
                    break;
                }
                e = new Exception();
            }
            case 2 -> {
                if (b == 0) {
                    return;
                }
                e = new Exception();
            }
            default -> {
                e = new Exception();
            }
        }
        throw e;
    }
}
