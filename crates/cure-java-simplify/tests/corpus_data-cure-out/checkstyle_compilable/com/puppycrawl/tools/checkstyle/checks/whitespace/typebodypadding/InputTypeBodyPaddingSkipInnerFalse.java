package com.puppycrawl.tools.checkstyle.checks.whitespace.typebodypadding;

class InputTypeBodyPaddingSkipInnerFalse {
    class InnerClass {
        private int a;
    }
    interface InnerInterface {
        void method();
    }
    enum InnerEnum {
        A, B
    }
    record InnerRecord(int a) {
        void method() {}
    }
    void method() {
        class LocalClass {
                    private int a;
                }
    }
    InputTypeBodyPaddingSkipInnerFalse() {
        class LocalClassInCtor {
                    private int a;
                }
    }
    static {
        class LocalClassInStaticInit {
                    private int a;
                }
    }
    {
        class LocalClassInInstanceInit {
                    private int a;
                }
    }
    Runnable r = () -> {
    class LocalClassInLambda {
                private int a;
            }
};
}
