package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

public class InputCommentsIndentationInSwitchBlockTwo {
    private static void foo1() {
        switch (1) {
            case 0:
            case 1:
                int b = 10;
            default:
        }
    }
    public void fooDotInCaseBlock() {
        int i = 0;
        String s = "";
        switch (i) {
            case -2:
                i++;
            case 0:
                s.indexOf("ignore");
            case -1:
                s.indexOf("no way");
            case 1:
            case 2:
                i--;
            case 3:
                {}
        }
        String breaks = "</table>";
    }
    public void foo2() {
        switch (1) {
            case 1:
            default:
        }
    }
    public void foo3() {
        switch (1) {
            case 1:
            default:
        }
    }
    public void foo4() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
    public void foo5() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
    public void foo6() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
}
