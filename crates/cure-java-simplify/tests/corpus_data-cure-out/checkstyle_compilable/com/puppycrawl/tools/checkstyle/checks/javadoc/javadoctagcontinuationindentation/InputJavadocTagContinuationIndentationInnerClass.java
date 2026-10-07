package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctagcontinuationindentation;

class InputJavadocTagContinuationIndentationInnerClass {
    class InnerClassWithAnnotations {
        String method(String aString) throws Exception {
            return "null";
        }
        String method1(String aString) throws Exception {
            return "null";
        }
        void method2(String aString) throws Exception {}
        void method3() throws Exception {}
        String method4() throws Exception {
            return "null";
        }
        String method5(String aString) {
            return "null";
        }
        String method6(String aString, int aInt, boolean aBoolean) throws Exception {
            return "null";
        }
    }
}
