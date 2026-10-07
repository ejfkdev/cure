package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableCheckSwitchExpressions2 {
    void foo1() throws Exception {
        Exception e;
        int a = (int) Math.random();
        int b = (int) Math.random();
        switch (a) {
            case 0 -> {
                e = new Exception();
            }
            default -> e = new Exception();
        }
        throw e;
    }
    void foo2() {
        int x = 0;
        int a = (int) Math.random();
        switch (a) {
            case 0:
                x = 1;
                break;
            default:
                x = 2;
                break;
        }
    }
}
