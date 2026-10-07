package com.puppycrawl.tools.checkstyle.checks.coding.defaultcomeslast;

public class InputDefaultComesLastSkipIfLastAndSharedWithCaseOne {
    void method(int i) {
        switch (i) {
            case 1:
            default:
                break;
            case 2:
                break;
        }
        switch (i) {
            case 1:
            default:
            case 2:
                break;
            case 3:
                break;
        }
        switch (i) {
            default:
            case 1:
                break;
            case 2:
                break;
        }
        switch (i) {
            case 0:
            default:
            case 1:
                break;
            case 2:
                break;
        }
        switch (i) {
            default:
            case 1:
                break;
            case 2:
                break;
        }
        switch (i) {
            case 1:
            default:
                break;
            case 2:
                break;
        }
        switch (i) {
            case 1:
            default:
                break;
            case 2:
                break;
            case 3:
                break;
        }
        switch (i) {
            case 1:
                break;
            default:
            case 2:
                break;
            case 3:
                break;
        }
        switch (i) {
            case 1:
                break;
            case 2:
            default:
                break;
            case 3:
                break;
        }
        switch (i) {
            case 1:
                break;
            default:
            case 3:
                break;
            case 4:
                break;
        }
        switch (i) {
            case 1:
                break;
            case 2:
                break;
            default:
            case 5:
            case 6:
                break;
        }
    }
}
