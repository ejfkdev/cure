package com.puppycrawl.tools.checkstyle.checks.naming.illegalidentifiername;

public class InputIllegalIdentifierNameUnnamedVariables {
    void m(Object o) {
        int _ = sideEffect();
        int __ = sideEffect();
        for (Integer _ : new int[0]) {}
        for (Integer BAD_ : new int[0]) {}
        for (Integer _BAD : new int[0]) {}
        try (var _ = lock()) {} catch (Exception _) {}
        switch (o) {
            case Integer _:
                {}
            default:
                {}
        }
        switch (o) {
            case R(int _ ,int _):
                {}
            default:
                {}
        }
    }
    int sideEffect() {
        return 0;
    }
    AutoCloseable lock() {
        return null;
    }
    record R(int x, int y) {
    }
}
