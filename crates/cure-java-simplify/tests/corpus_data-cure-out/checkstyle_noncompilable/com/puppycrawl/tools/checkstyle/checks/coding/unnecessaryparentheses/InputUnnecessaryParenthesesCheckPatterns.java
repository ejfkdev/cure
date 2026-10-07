package com.puppycrawl.tools.checkstyle.checks.coding.unnecessaryparentheses;

public class InputUnnecessaryParenthesesCheckPatterns {
    void method() {
        Object o = "";
    }
    record Rectangle(int x, int y) {
    }
}
