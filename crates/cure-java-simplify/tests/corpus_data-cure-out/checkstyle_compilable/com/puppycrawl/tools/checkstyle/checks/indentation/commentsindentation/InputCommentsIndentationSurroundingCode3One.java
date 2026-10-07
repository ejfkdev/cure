package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

import java.util.Arrays;

public class InputCommentsIndentationSurroundingCode3One {
    private void foo1() {
        int b = 10;
    }
    private void foo2() {
        double d;
        boolean bb;
        boolean x;
    }
    private void foo3() {
        int b = 3;
    }
    private static void com() {
        boolean b = true;
    }
    private static final String[][] mergeMatrix = {{""}, {"", ""}, {"NEVER", "UNKNOWN", "NEVER"}, {"UNKNOWN", "UNKNOWN", "UNKNOWN", "UNKNOWN"}};
    private void foo4() {
        if (!Arrays.equals(new String[] {""}, new String[] {""})) {}
    }
    private static void l() {}
    public void foid5() {
        "".toString().toString().toString();
    }
    public void foo6() {
        String someStr = "";
    }
    public void foo7() {
        String someStr = "";
    }
}
