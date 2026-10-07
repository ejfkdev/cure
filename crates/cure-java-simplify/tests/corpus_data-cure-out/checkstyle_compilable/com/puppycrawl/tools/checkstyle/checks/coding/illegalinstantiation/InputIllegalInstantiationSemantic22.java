package com.puppycrawl.tools.checkstyle.checks.coding.illegalinstantiation;

import java.io.*;
import java.awt.Dimension;
import java.awt.Color;
import java.util.*;

public class InputIllegalInstantiationSemantic22 {
    public void triggerEmptyBlockWithoutBlock() {}
    {}
    public class EqualsVsHashCode5 {
        public <A> boolean equals(int a) {
            return a == 1;
        }
    }
    public class EqualsVsHashCode6 {
        public <A> boolean equals(Comparable<A> a) {
            return true;
        }
    }
    private class InputBraces {
    }
    private class InputModifier {
    }
    private void method() {
        Boolean[] array = new Boolean[3];
        Object object = new @Interned Object();
        Map<Class<?>, Boolean> x = new HashMap<Class<?>, Boolean>();
    }
    @java.lang.annotation.Target(java.lang.annotation.ElementType.TYPE_USE) @interface Interned {
    }
}
