package com.puppycrawl.tools.checkstyle.checks.modifier.modifierorder;

public class InputModifierOrderCustomOrderTwo {
    public @Deprecated int a;
    public @Deprecated void method() {}
    @Deprecated private int c;
    @Deprecated protected int e;
    private @MethodsAnnotation void foo11() {}
    final int f = 10;
}

@interface MethodsAnnotation {
}
