package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

public class InputWhitespaceAroundSwitchCasesParensWithAllowEmptySwitchBlockStatements {
    void m(int k) {
        switch (k) {
            case 1:
                System.out.println("1");
            case 2:
                {}
            case 3:
            case 4:
                {}
            default:
                {}
        }
    }
    void m2(int k) {
        switch (k) {
            case 1 -> System.out.println("1");
            case 2 -> {}
            case 3 -> {}
            default -> {}
        }
    }
    void m3(int k) {
        switch (k) {
            case 1 -> {
                System.out.println("1");
            }
            default -> {}
        }
    }
    void m4(int k) {
        switch (k) {
            case 1:
                {
                    System.out.println("1");
                }
            default:
                {}
        }
    }
    void m5(int k) {
        System.out.println("1");
    }
    void m6(int k) {
        switch (k) {
            case 1:
                {}
                {}
            case 2:
                {}
                {}
            case 3:
                {}
            case 4:
                {}
            case 5:
                {}
            case 6:
                {}
            case 7:
                {}
                break;
        }
    }
}
