package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationInvalidArrayIndexIndent {
    void test() {
        int[] array = new int[10];
        array[1] = 0;
    }
}
