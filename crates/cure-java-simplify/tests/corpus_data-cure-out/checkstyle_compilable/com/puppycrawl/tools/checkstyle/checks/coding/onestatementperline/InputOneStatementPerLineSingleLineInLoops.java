package com.puppycrawl.tools.checkstyle.checks.coding.onestatementperline;

public class InputOneStatementPerLineSingleLineInLoops {
    private int one = 0;
    private int two = 0;
    public void doIllegal2() {
        one = 1;
        two = 2;
    }
    public void doStringBuffer() {
        StringBuffer sb = new StringBuffer();
        sb.append("test ");
        sb.append("test2 ").append("test3 ");
        appendToSpringBuffer(sb, "test4");
    }
    private void appendToSpringBuffer(StringBuffer sb, String text) {
        sb.append(text);
    }
    int a;
    int b;
    int c;
    int d;
    int e = 1;
    int f = 2;
    int g = 1;
    int h = 2;
    private void foo() {
        int var1 = 1;
        int var2 = 2;
        var1++;
        var2++;
        Object obj1 = new Object();
        Object obj2 = new Object();
    }
}
