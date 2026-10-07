package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

public class InputJavadocMethod_1379666 {
    public void ok() throws BadStringFormat {}
    public void error1() throws java.lang.Exception {}
    public void error2() throws InputJavadocMethod_1379666.BadStringFormat {}
    public static class BadStringFormat extends Exception {
        BadStringFormat(String s) {
            super(s);
        }
    }
}
