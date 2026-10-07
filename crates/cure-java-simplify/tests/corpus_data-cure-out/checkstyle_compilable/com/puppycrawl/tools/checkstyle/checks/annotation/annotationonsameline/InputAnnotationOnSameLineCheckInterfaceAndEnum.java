package com.puppycrawl.tools.checkstyle.checks.annotation.annotationonsameline;

import java.util.List;

@Ann
@Ann2 interface TestInterface {
    @Ann
    @Ann2 Integer getX();
}

public @Ann
@Ann2 class InputAnnotationOnSameLineCheckInterfaceAndEnum implements @Ann
        @Ann2 TestInterface {
    @Ann
    @Ann2 private Integer x;
    (0);

        // violation below "Annotation 'Ann' should be on the same line with its target."
    private List<@Ann
            @Ann2 Integer> integerList;
    @Ann
    @Ann2 enum TestEnum {
        A1, A2
    }
    @Ann
    @Ann2 public InputAnnotationOnSameLineCheckInterfaceAndEnum() {}
    @Ann
    @Ann2 public void setX(@Ann
            // violation below "Annotation 'Ann' should be on the same line with its target."
            @Ann2 int x) throws @Ann
                    @Ann2 Exception {
        this.getXAs();
        this.x = x;
    }
    @Override public Integer getX() {
        return (Integer) x;
    }
    public <@Ann T> T getXAs() {
        return (T) x;
    }
}

@Ann
@Ann2 @interface TestAnnotation {
    @Ann
        @Ann2 int x();
}
