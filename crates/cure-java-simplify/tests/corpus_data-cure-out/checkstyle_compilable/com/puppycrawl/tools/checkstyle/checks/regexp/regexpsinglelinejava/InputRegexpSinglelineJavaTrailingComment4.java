package com.puppycrawl.tools.checkstyle.checks.regexp.regexpsinglelinejava;

public class InputRegexpSinglelineJavaTrailingComment4 {
    int i;
    int j;
    void method1() {
        Runnable r = new Runnable() {
                public void run() {
                }
            };
    }
    void method2(long ms) {}
    final static public String NAME = "Some Name";
}
