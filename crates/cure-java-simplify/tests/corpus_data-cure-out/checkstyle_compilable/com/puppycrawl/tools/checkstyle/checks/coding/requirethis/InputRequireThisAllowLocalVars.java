package com.puppycrawl.tools.checkstyle.checks.coding.requirethis;

class InputRequireThisAllowLocalVars {
    private String s1 = "foo1";
    String s2 = "foo2";
    InputRequireThisAllowLocalVars() {
        String s2 = "bar2";
    }
    public int getS1() {
        return 1;
    }
    public String getS1(String param) {
        String s1 = param;
        s1 += s1;
        return s1;
    }
    String getS2() {
        String s2 = null;
        s2 += s2;
        return "return";
    }
    String getS2(String s2) {
        s2 = null;
        return s2;
    }
    String getS2(int a) {
        String s2 = " ";
        s2 += s2;
        return s1;
    }
}
