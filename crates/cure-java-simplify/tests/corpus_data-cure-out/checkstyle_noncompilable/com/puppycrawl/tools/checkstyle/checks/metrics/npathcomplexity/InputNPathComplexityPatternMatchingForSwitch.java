package com.puppycrawl.tools.checkstyle.checks.metrics.npathcomplexity;

public class InputNPathComplexityPatternMatchingForSwitch {
    void testPatternWithRule(Object o) {
        switch (o) {
            case Integer _ -> {}
            case String _ -> {}
            default -> {}
        }
    }
    void testPatternWithStatement(Object o) {
        switch (o) {
            case Integer _:
                {}
            case String _:
                {}
            default:
                {}
        }
    }
    void testRecordPatternWithRule(Object o) {
        switch (o) {
            case A(int x) -> {}
            case B(String s) -> {}
            default -> {}
        }
    }
    void testRecordPatternWithStatement(Object o) {
        switch (o) {
            case A(int x):
                {}
                break;
            case B(String s):
                {}
            default:
                {}
        }
    }
    void testGuardsInRule(Object o) {
        switch (o) {
            case Integer i when i > 0 -> {}
            case String s when s.length() > 0 -> {}
            default -> {}
        }
    }
    void testGuardsInStatement(Object o) {
        switch (o) {
            case Integer i when i > 0:
                {}
                break;
            case String s when s.length() > 0:
                {}
            default:
                {}
        }
    }
    void testMultiCaseLabelWithRule(Object o) {
        switch (o) {
            case Integer _, String _, Double _ -> {}
            default -> {}
        }
    }
    void testMultiCaseLabelWithStatement(Object o) {
        switch (o) {
            case Integer _:
            case String _:
            case Double _:
                {}
            default:
                {}
        }
    }
    void testExprInRule(int x) {
        switch (x) {
            case 1 -> {}
            case 2 -> {}
            default -> {}
        }
    }
    void testExprInStatement(int x) {
        switch (x) {
            case 1:
                {}
            case 2:
                {}
            default:
                {}
        }
    }
    record A(int x) {
    }
    record B(String s) {
    }
}
