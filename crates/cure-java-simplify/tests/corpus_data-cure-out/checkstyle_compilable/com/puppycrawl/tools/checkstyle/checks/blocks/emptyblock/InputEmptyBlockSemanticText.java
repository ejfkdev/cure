package com.puppycrawl.tools.checkstyle.checks.blocks.emptyblock;

import java.io.*;
import java.awt.Dimension;
import java.awt.Color;

class InputEmptyBlockSemanticText {
    static {
        Boolean x = new Boolean(true);
    }
    {
        Boolean x = new Boolean(true);
        Boolean[] y = {Boolean.TRUE, Boolean.FALSE};
    }
    Boolean getBoolean() {
        return new java.lang.Boolean(true);
    }
    void exHandlerTest() {}
    private static final long IGNORE = 666l + 666L;
    public class EqualsVsHashCode1 {
        public boolean equals(int a) {
            return a == 1;
        }
    }
    {}
    private class InputBraces {
    }
    synchronized void foo() {
        synchronized (this) {}
        synchronized (Class.class) {
            synchronized (new Object()) {}
        }
    }
    static {}
    static {}
}
