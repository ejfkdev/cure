package com.puppycrawl.tools.checkstyle.checks.annotation.annotationusestyle;

public class InputAnnotationUseStyleNoTrailingComma {
    @Test2(value={(false) ? "" : "foo"}, more={(true) ? "" : "bar"}) enum P {
        L, Y
    }
}

@interface Test2 {
    String[] value();
    String[] more() default {};
}
