package com.puppycrawl.tools.checkstyle.checks.naming.localfinalvariablename;

public class InputLocalFinalVariableNameUnnamedVariables {
    void testTryWithResource(Object obj) {
        try (var _ = lock()) {} catch (Exception _) {}
        try (var __ = lock()) {} catch (Exception __) {}
    }
    void testEnhancedForLoop() {
        for (var _ : new int[0]) {}
        for (var __ : new int[0]) {}
        for (var _BAD : new int[0]) {}
    }
    void testLocalVariable(Object obj) {
        var _BAD = obj;
    }
    public AutoCloseable lock() {
        return null;
    }
}
