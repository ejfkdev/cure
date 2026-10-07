package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacebeforeemptybody;

public class InputWhitespaceBeforeEmptyBodyLambda {
    interface Foo {
        void foo();
    }
    interface Bar {
        int bar();
    }
    void test() {
        Bar c = () -> 0;
    }
}
