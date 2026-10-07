package com.puppycrawl.tools.checkstyle.checks.annotation.annotationusestyle;

public class InputAnnotationUseStyleWithTrailingComma {
    @Test(value={(false) ? "" : "foo",}, more={(true) ? "" : "bar",}) enum P {
        L, Y
    }
}

@interface Test {
    String[] value();
    String[] more() default {};
}
