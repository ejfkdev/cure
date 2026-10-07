package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

import java.util.Arrays;

public class InputCommentsIndentationSurroundingCode3One {
    private void foo1() {}
    private void foo2() {}
    private void foo3() {}
    private static void com() {}
    private static final String[][] mergeMatrix = {{""}, {"", ""}, {"NEVER", "UNKNOWN", "NEVER"}, {"UNKNOWN", "UNKNOWN", "UNKNOWN", "UNKNOWN"}};
    private void foo4() {
        if (!Arrays.equals(new String[] {""}, new String[] {""})) {}
    }
    private static void l() {}
    public void foid5() {
        "".toString().toString().toString();
    }
    public void foo6() {}
    public void foo7() {}
}
