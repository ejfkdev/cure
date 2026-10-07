package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationCheckSwitchExpressionNewLine {
    private B map(A a) {
        return switch (a) {
            case one, two, three, four -> B.one;
            default -> B.two;
        };
    }
    private A map(B a) {
        return switch (a) {
            case one, two, three, four -> A.one;
            default -> A.two;
        };
    }
    enum A {
        one, two, three, four, five
    }
    enum B {
        one, two, three, four, five
    }
}
