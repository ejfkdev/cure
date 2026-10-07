package com.puppycrawl.tools.checkstyle.checks.blocks.needbraces;

public class InputNeedBracesTestCaseDefaultNoSingleLine {
    public String aMethod(int val) {
        switch (val) {
            default:
            case 0:
            case -1:
                break;
            case -2:
                Math.random();
        }
        switch (val) {
            default:
                break;
        }
        switch (val) {
            default:
                Math.random();
        }
        switch (val) {
            case 1:
                {}
            default:
        }
        switch (val) {
            case 0:
                {
                    return "zero";
                }
            case 1:
                {
                    return "one";
                }
            default:
                {
                    return "non-binary";
                }
        }
    }
}
