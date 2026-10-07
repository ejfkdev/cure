package com.puppycrawl.tools.checkstyle.checks.coding.multiplevariabledeclarations;

public class InputMultipleVariableDeclarationsUnnamedVariables {
    void test() {
        int _ = sideEffect();
        int _ = sideEffect();
        int _ = sideEffect();
        int a = sideEffect();
        int _ = sideEffect();
        int _ = sideEffect();
        int _ = sideEffect();
        int b = sideEffect();
    }
    int sideEffect() {
        return 0;
    }
}
