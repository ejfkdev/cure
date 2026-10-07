package com.puppycrawl.tools.checkstyle.checks.coding.illegalthrows;

public class InputIllegalThrowsIgnoreMethodNames {
    public void method() throws NullPointerException {}
    public java.lang.Throwable methodOne() throws RuntimeException {
        return null;
    }
    public void methodTwo() throws java.lang.RuntimeException, java.lang.Error {}
    public void finalize() throws Throwable {}
}
