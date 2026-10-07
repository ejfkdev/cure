package com.puppycrawl.tools.checkstyle.checks.whitespace.emptylineseparator;

import org.junit.Assert;
import org.junit.Test;

public class InputEmptyLineSeparatorInsideClassMembers {
    public void foo(int a) {}
    @Test
    public void testFoo() {
        new InputEmptyLineSeparatorInsideClassMembers().foo(10);
        Assert.assertFalse(false);
    }
}
