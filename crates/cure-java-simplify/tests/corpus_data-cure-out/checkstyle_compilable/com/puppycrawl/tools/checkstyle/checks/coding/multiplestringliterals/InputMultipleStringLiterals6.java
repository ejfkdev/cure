package com.puppycrawl.tools.checkstyle.checks.coding.multiplestringliterals;

public class InputMultipleStringLiterals6 {
    String m = "StringContents";
    String m1 = "SingleString";
    String m2 = "DoubleString" + "DoubleString";
    String m3 = "" + "";
    String m4 = "" + "";
    String debugStr = ", , " + ", ";
    void method1() {
        System.identityHashCode("StringContents");
        String a2 = "StringContents";
    }
    @SuppressWarnings("unchecked") void method2() {}
    @SuppressWarnings("unchecked") void method3() {}
    @SuppressWarnings("unchecked") void method4() {}
    @SuppressWarnings("unchecked") void method5() {}
}
