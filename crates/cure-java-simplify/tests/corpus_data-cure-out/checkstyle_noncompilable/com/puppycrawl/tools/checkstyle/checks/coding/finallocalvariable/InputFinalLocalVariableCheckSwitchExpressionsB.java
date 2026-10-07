package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableCheckSwitchExpressionsB {
    void foo3() throws Exception {
        Exception e;
        int a = (int) Math.random();
        int b = (int) Math.random();
        switch (a) {
            case 0 -> e = new Exception();
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
            default -> e = new Exception();
        }
        throw e;
    }
    void foo4() throws Exception {
        Exception e;
        int a = (int) Math.random();
        int b = (int) Math.random();
        switch (a) {
            case 0 -> e = new Exception();
            case 1 -> System.out.println("test!");
            default -> System.out.println("Exception!");
        }
    }
}
