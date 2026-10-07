package com.puppycrawl.tools.checkstyle.checks.indentation.openjdkmethodthrowsalignment;

public class InputOpenjdkMethodThrowsAlignment {
    void testMethod(String aString, int bInt) throws InterruptedException {}
    void testMethod1(String aString, int bInt, int cInt) throws InterruptedException {}
    void testMethod2(String aString, int bInt, int cInt) throws InterruptedException {}
    void testMethod3(String aString, int bInt) throws InterruptedException {}
    void met(int a, int b) throws Exception {}
    void met2(int aInt, int bInt) throws InterruptedException {}
    void met3(int aInt, int bInt) throws InterruptedException {}
    void noThrowsWrapped(String aString, int bInt) {}
    void singleLine(int aInt) throws InterruptedException {}
    InputOpenjdkMethodThrowsAlignment(String aString, int bInt) throws InterruptedException {}
    InputOpenjdkMethodThrowsAlignment(String aString, int bInt, int cInt) throws InterruptedException {}
    InputOpenjdkMethodThrowsAlignment(int aInt, int bInt) throws InterruptedException {}
    void multipleExceptions(String aString, int bInt) throws InterruptedException, IllegalArgumentException {}
}
