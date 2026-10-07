package com.puppycrawl.tools.checkstyle.checks.whitespace.operatorwrap;

public class InputOperatorWrapGuardedPatterns {
    String typeGuardAfterParenthesizedTrueIfStatement1(Object o) {
        return o != null && o instanceof Integer i && i == 0 ? "true" : o != null && o instanceof Integer i && i == 2 && (o = i) != null ? "second" : "any";
    }
    String typeGuardAfterParenthesizedTrueIfStatement2(Object o) {
        return o != null && o instanceof Integer i && i == 0 ? "true" : o != null && o instanceof Integer i && i == 2 && (o = i) != null ? "second" : "any";
    }
}
