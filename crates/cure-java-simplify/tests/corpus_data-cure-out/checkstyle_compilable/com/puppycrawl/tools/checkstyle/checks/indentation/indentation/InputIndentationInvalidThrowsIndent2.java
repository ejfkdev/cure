package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public abstract class InputIndentationInvalidThrowsIndent2 {
    public void m1() throws Exception {}
    public void m2() throws Exception {}
    public void m3() throws Exception, NullPointerException {}
    public void m4() throws Exception {}
    public abstract void m5() throws Exception;
    public void m6() throws Exception {}
    public void m7() throws Exception, NullPointerException {}
    double[] m8() throws Exception {
        return null;
    }
    public InputIndentationInvalidThrowsIndent2() throws Exception {}
    @TestAnnotation                                                         //indent:1 exp:1
 public                                                                  //indent:1 exp:1
    static void m9() throws Exception {}
}

@interface TestAnnotation {
}
