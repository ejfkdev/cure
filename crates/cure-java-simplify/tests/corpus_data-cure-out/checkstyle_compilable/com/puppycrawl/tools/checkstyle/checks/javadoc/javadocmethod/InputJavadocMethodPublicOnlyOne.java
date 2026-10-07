package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

public class InputJavadocMethodPublicOnlyOne {
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
    private int mSize;
    int mLen;
    protected int mDeer;
    public int aFreddo;
    private InputJavadocMethodPublicOnlyOne(int aA) {}
    InputJavadocMethodPublicOnlyOne(String aA) {}
    protected InputJavadocMethodPublicOnlyOne(Object aA) {}
    public InputJavadocMethodPublicOnlyOne(Class<Object> aA) {}
    private void method(int aA) {}
    void method(Long aA) {}
    protected void method(Class<Object> aA) {}
    public void method(StringBuffer aA) {}
}
