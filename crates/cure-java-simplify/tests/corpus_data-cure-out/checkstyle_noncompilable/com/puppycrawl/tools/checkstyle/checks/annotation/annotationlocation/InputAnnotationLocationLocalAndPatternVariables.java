package com.puppycrawl.tools.checkstyle.checks.annotation.annotationlocation;

public class InputAnnotationLocationLocalAndPatternVariables {
    void test(Object obj) {
        @SuppressWarnings("deprecation") int x = 5;
        @SuppressWarnings("deprecation") int _ = 5;
        switch (obj) {
            case ColoredPoint( @Deprecated int y,_,_) -> {}
            default -> {}
        }
    }
    record ColoredPoint(int p, int x, int c) {
    }
}
