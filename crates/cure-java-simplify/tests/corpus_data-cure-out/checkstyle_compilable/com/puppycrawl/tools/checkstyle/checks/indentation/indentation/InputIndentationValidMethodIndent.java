package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

import java.util.Arrays;

public class InputIndentationValidMethodIndent extends Object {
    public InputIndentationValidMethodIndent() {}
    private InputIndentationValidMethodIndent(boolean test) {}
    private InputIndentationValidMethodIndent(boolean test, boolean test2) {}
    private InputIndentationValidMethodIndent(boolean test, boolean test2, boolean test3) {}
    public InputIndentationValidMethodIndent(int dummy) {}
    public void method2() {}
    public void method2(int x, int y, int w, int h, int x1, int y1, int w1, int h1) {}
    public void method3(int x, int y, int w, int h, int x1, int y1, int w1, int h1) {
        System.getProperty("foo");
    }
    public void method5() {}
    private int[] getArray() {
        return new int[] {1};
    }
    private void indexTest() {
        getArray()[0] = 2;
    }
    @SuppressWarnings(                                                         //indent:4 exp:4
		value=""                                                               //indent:8 exp:8
	)                                                                          //indent:4 exp:4
	public void testStartOfSequence() {}
}
