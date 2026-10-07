package com.puppycrawl.tools.checkstyle.checks.annotation.annotationonsameline;

public class InputAnnotationOnSameLinePatternVariables {
    void test(Object o) {
        @Deprecated int _ = 0;

               // violation below 'Annotation 'Deprecated' should be on the same line with its target.'
        @Deprecated
               int _ = 0;
    }
    void test2(Object o) {
        switch (o) {
            case ColoredPoint(@Deprecated
                              int x,
                              @Deprecated
                              int y,
                              @Deprecated
                              String color) when x >= 0 -> {}
            case ColoredPoint(@Deprecated
                              int x,
                              @Deprecated
                              int _,
                              @Deprecated
                              String color) -> {}
            default -> {}
        }
    }
    record ColoredPoint(@Deprecated
                        int x,
                        @Deprecated
                        int y,
                        @Deprecated
                        String color) {
    }
}
