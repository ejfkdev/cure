package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadoctype;

public class InputMissingJavadocTypePublicOnly2One {
    private interface InnerInterface {
        String CONST = "InnerInterface";
        void method();
        class InnerInnerClass {
            private int mData;
            private InnerInnerClass() {
                Runnable r = new Runnable() {
                    public void run() {};
                };
            }
            void method2() {
                Runnable r = new Runnable() {
                    public void run() {};
                };
            }
        }
    }
    private class InnerClass {
        private int mDiff;
        void method() {}
    }
}
