package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadoctype;

public class InputMissingJavadocTypeEnum {
    enum Test {
        Test{
            @Override
            <T> void method(
                    T value) { }
        };
        abstract <T> void method(T value);
    }
    enum Test2 {
        TEST_2 {
            @Override
            <T> void method(
                    T value) { }
        };
        abstract <T> void method(T value);
    }
    static class A {
    }
}
