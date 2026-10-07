package com.puppycrawl.tools.checkstyle.checks.coding.illegaltype;

import static com.puppycrawl.tools.checkstyle.checks.coding.illegaltype.InputIllegalType.SomeStaticClass;
import java.lang.String;

public class InputIllegalTypeTestStaticImports {
    private boolean foo(String s) {
        return true;
    }
    SomeStaticClass staticClass;
    private static SomeStaticClass foo1() {
        return null;
    }
    private static void foo2(SomeStaticClass s) {}
}
