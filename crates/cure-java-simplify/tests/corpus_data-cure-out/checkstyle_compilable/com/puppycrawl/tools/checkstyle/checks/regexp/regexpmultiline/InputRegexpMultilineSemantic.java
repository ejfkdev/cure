package com.puppycrawl.tools.checkstyle.checks.regexp.regexpmultiline;

import java.awt.*;
import java.io.ByteArrayOutputStream;
import java.io.File;

class InputRegexpMultilineSemantic {
    static {
        Boolean x = new Boolean(true);
    }
    {
        Boolean x = new Boolean(true);
        Boolean[] y = {Boolean.TRUE, Boolean.FALSE};
    }
    Boolean getBoolean() {
        return new Boolean(true);
    }
    void otherInstantiations() {
        Object o1 = new InputBraces();
        Object o2 = new InputModifier();
        ByteArrayOutputStream s = new ByteArrayOutputStream();
        File f = new File("/tmp");
        Dimension dim = new Dimension();
        Color col = new Color(0, 0, 0);
    }
    void exHandlerTest() {
        try {} catch (IllegalStateException emptyCatchIsAlwaysAnError) {} catch (NullPointerException ex) {} catch (ArrayIndexOutOfBoundsException ex) {} catch (NegativeArraySizeException ex) {} catch (UnsupportedOperationException handledException) {
            System.out.println(handledException.getMessage());
        } catch (SecurityException ex) {} catch (StringIndexOutOfBoundsException ex) {} catch (IllegalArgumentException ex) {}
    }
    private static final long IGNORE = 666l + 666L;
    public class EqualsVsHashCode1 {
        public boolean equals(int a) {
            return a == 1;
        }
    }
    public class EqualsVsHashCode2 {
        public boolean equals(String a) {
            return true;
        }
    }
    public class EqualsVsHashCode3 {
        public boolean equals(Object a) {
            return true;
        }
        public int hashCode() {
            return 0;
        }
    }
    public class EqualsVsHashCode4 {
        ByteArrayOutputStream bos1 = new ByteArrayOutputStream() {
            public boolean equals(Object a) // don't flag
            {
                return true;
            }

            public int hashCode()
            {
                return 0;
            }
        };
        ByteArrayOutputStream bos2 = new ByteArrayOutputStream() {
            public boolean equals(Object a) // flag
            {
                return true;
            }
        };
    }
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
    synchronized void foo() {
        synchronized (this) {}
        synchronized (Class.class) {
            synchronized (new Object()) {}
        }
    }
    static {
        int a = 0;
    }
    static {}
}
