package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctype;

class InputJavadocTypeTags1 {
    private int mMissingJavadoc;
    void method1() {}
    void method2() {}
    int method3() {
        return 3;
    }
    int method4(int aOne) {
        return aOne;
    }
    void method5() throws Exception {}
    void method6() throws Exception {}
    void method7() throws Exception, NullPointerException {}
    void method8(int aOne) {}
    void method9(int aOne) {}
    void method10(int aOne, int aTwo) {}
    void method11() {}
    int method12() {
        return 0;
    }
}
