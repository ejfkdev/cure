package com.puppycrawl.tools.checkstyle.checks.coding.innerassignment;

public class InputInnerAssignmentSwitchAndSwitchExpression {
    public void test1(int mode) {
        int x = 0;
        switch (mode) {
            case 2 -> {
                x = 2;
            }
            case 1 -> x = 1;
        }
    }
    public void test2(int mode) {
        int x = 0;
        int y = 0;
        switch (mode) {
            case 2, 4, 6 -> {
                x = 2;
            }
            case 1, 3, 5 -> {
                x = y = 1;
            }
            case 0, 7, 8 -> x = 1;
        }
    }
    public void test3(int mode) {
        int x;
        x = switch (mode) {
            case 2 -> {
                yield x = 2;
            }
            case 1 -> x = 1;
            default -> x = 0;
        };
    }
    public void test4(int mode) {
        int x;
        x = switch (mode) {
            case 2, 4, 6 -> {
                yield x = 2;
            }
            case 1, 3, 5 -> x = 1;
            default -> x = 0;
        };
    }
    public void test5(String operation) {
        boolean innerFlag;
        boolean flag;
        switch (operation) {
            case "Y" -> flag = innerFlag = true;
            case "N" -> {
                flag = innerFlag = false;
            }
        }
    }
    public void test6(int mode) {
        int x = 0;
        switch (mode) {
            case 2:
                {
                    x = 2;
                }
            case 1:
                x = 1;
        }
    }
    public void test7(int mode) {
        int x = 0;
        switch (mode) {
            case 0:
            case 1:
            case 2:
                {
                    x = 2;
                }
            case 4:
            case 5:
                x = 1;
        }
    }
    public void test8(int mode) {
        int x = 4;
        System.out.println(switch (x) {
            case 1 -> x = 1;
            case 2 -> {
                yield x = 2;
            }
            default -> x = 3;
        });
    }
}
