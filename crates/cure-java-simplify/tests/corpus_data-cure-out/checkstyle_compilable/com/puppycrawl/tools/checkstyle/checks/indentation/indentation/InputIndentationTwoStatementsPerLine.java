package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationTwoStatementsPerLine {
    int var6 = 5;
    int var7 = 6, var8 = 5;
    public void method() {
        long_lined_label:
            {}
    }
    static final void method2() {}
}
