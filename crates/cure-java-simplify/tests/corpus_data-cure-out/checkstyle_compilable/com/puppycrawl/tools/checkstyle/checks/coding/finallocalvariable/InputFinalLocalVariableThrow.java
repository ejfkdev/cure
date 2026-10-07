package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableThrow {
    void terminatingBranchBad(boolean condition) {
        int value;
        if (condition) {
            value = 1;
            throw new IllegalArgumentException();
        }
        value = 2;
    }
    void terminatingBranchGood(boolean condition) {
        int value;
        if (condition) {
            value = 1;
            throw new IllegalArgumentException();
        }
        value = 2;
    }
    void nestedBranchBad(boolean first, boolean second) {
        int value;
        if (first) {
            if (second) {
                value = 1;
                throw new IllegalArgumentException();
            }
            value = 2;
            throw new IllegalStateException();
        }
        value = 3;
    }
    void nestedBranchGood(boolean first, boolean second) {
        int value;
        if (first) {
            if (second) {
                value = 1;
                throw new IllegalArgumentException();
            }
            value = 2;
            throw new IllegalStateException();
        }
        value = 3;
    }
    void repeatedAssignment(boolean condition) {
        int value;
        if (condition) {
            value = 2;
            throw new IllegalArgumentException();
        }
        value = 3;
    }
    void assignedBeforeBranch(boolean condition) {
        int value = 2;
        if (condition) {
            value = 1;
            throw new IllegalArgumentException();
        }
    }
    void assignedAfterBranch(boolean condition) {
        int value;
        if (condition) {
            value = 1;
            throw new IllegalArgumentException();
        }
        value = 3;
    }
}
