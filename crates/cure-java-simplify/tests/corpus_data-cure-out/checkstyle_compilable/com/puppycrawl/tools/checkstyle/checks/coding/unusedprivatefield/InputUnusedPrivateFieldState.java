package com.puppycrawl.tools.checkstyle.checks.coding.unusedprivatefield;

class InputUnusedPrivateFieldState {
    private int a;
}

class SecondClass extends InputUnusedPrivateFieldState {
    private int used;
    int method() {
        return used;
    }
}
