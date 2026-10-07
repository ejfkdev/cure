package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class InputRegressionJavaClass1 {
    public int f1;
    private int f2;
    protected int f3;
    int f4;
    static int f5;
    final int f6;
    volatile int f7;
    transient int f8;
    Object f9;
    static {}
    public InputRegressionJavaClass1() {
        f6 = 0;
    }
    public InputRegressionJavaClass1(int i) {
        this.f6 = i;
    }
    public InputRegressionJavaClass1(float f) {
        this((int) f);
    }
    InputRegressionJavaClass1(double a) throws Exception {
        f6 = 0;
    }
    native void m1();
    void m2() {}
    synchronized void m4() {}
    strictfp void m5() {}
    public int[] m6() {
        return null;
    }
    public int[] m7() {
        return null;
    }
    public void m10(String p1) {}
    public void m11(final String p1) {}
    public void m12(String[] p1) {}
    public void m13(String[][] p1) {}
    public void m14(String p1, String p2) {}
    public void m15(String... p1) {}
    public void m16(String[]... p1) {}
    public void m17(int p1, String[]... p2) {}
    public void m18() throws Exception {}
    public void m19() throws IOException, Exception {}
    public <T_$> T_$ m20() {
        return null;
    }
    public <$_T> $_T m21() {
        return null;
    }
    public <T extends Enum<T>> void m22() {}
    public <T> void m23() {}
    public <T extends RuntimeException & java.io.Serializable> void m24() {}
    @SuppressWarnings({})
    public void m50() {}
    @SuppressWarnings({"1"})
    public void m51() {}
    @SuppressWarnings({"1","2"})
    public void m52() {}
    @SuppressWarnings(value={"1"})
    public void m53() {}
    @SuppressWarnings(value={"1",})
    public void m54() {}
    @SuppressWarnings(value={"1","2"})
    public void m55() {}
    @InputRegressionJavaAnnotation1(m1="1", m2="2")
    public void m56() {}
    @ComplexAnnotation({
            @InputRegressionJavaAnnotation1(m1 = "1", m2 = ""),
            @InputRegressionJavaAnnotation1(m1 = "1", m2 = "")
    })
    public void m57() {}
    public void m58(@Deprecated String s) {}
    public void m59(final @Deprecated List l) {}
    {}
    public void instructions() throws Exception {
        boolean b = Math.random() > 0;
        byte vbyte;
        boolean vboolean;
        char vchar;
        short vshort;
        int vint;
        long vlong;
        float vfloat;
        double vdouble;
        int[] varray;
        int[] varray2;
        boolean test1 = true;
        String vstring;
        List<String> vlist;
        Map<String, String[]> vmap;
        int[] test2 = {};
        List<char[]> test3;
        Class<?> test4;
        List<? extends InputRegressionJavaClass1> test5;
        List<? extends List<Object>> test6;
        List<? extends List<List<Object>>> test7;
        List<? extends int[]> test8;
        List<? super InputRegressionJavaClass1> test9;
        vchar = '\\';
        vlong = 0XABCDEFL;
        vfloat = 0x2__3_34.4___AFP00_00f;
        vdouble = 0x.1___AFp1;
        vboolean = f9 instanceof Object;
        vint = 0;
        vint *= 1;
        vint /= 1;
        vint %= 1;
        vint &= 1;
        vint |= 1;
        vint ^= 1;
        vint <<= 1;
        vint >>= 1;
        vint >>>= 1;
        vint++;
        vint--;
        ++vint;
        --vint;
        String[] arrayinit = {};
        String[] arrayinit2 = {""};
        String[] arrayinit3 = {"", ""};
        varray = new int[] {};
        varray = new int[] {0};
        varray = new int[] {0, 1};
        varray = new int[5];
        vlist = new ArrayList<String>();
        vmap = new HashMap<String, String[]>();
        Object anonymous = new InputRegressionJavaClass1() {};
        this.f1 = 0;
        test_label1:
            {}
        if (b) {
            for (; ; ) ;
        }
        if (b) {
            for (; ; ) {}
        }
        for (int i = 0; i < 1; i++) {}
        for (int i = 0, j = 0; i < 1; i++, j += 2) {}
        for (int value : new int[] {}) ;
        for (String s : new String[] {}) ;
        for (String s : new String[] {}) ;
        if (b) {
            while (true) ;
        }
        if (b) {
            while (true) {}
        }
        do {} while (false);
        synchronized (f9) {}
        switch (0) {
            case 1:
            case 0:
                break;
            default:
                break;
        }
        try {
            if (b) {
                throw new IOException();
            }
            if (b) {
                throw new ArrayIndexOutOfBoundsException();
            }
            throw new Exception();
        } catch (IOException | ArrayIndexOutOfBoundsException e) {} catch (Exception e) {}
        try (BufferedReader br = new BufferedReader(new InputStreamReader(null, "utf-8"))) {}
        try (BufferedReader br1 = new BufferedReader(new InputStreamReader(null, "utf-8")); BufferedReader br2 = new BufferedReader(new InputStreamReader(null, "utf-8"))) {}
        test4 = List[].class;
        test4 = boolean[].class;
        varray[0] = 0;
        for (String[] s : new String[][] {{}}) ;
        for (Map.Entry<String, String[]> e : vmap.entrySet()) {}
        for (; ; ) {
            break;
        }
        test_label2:
            for (; ; ) {
                break test_label2;
            }
        if (b) {
            for (; ; ) {}
        }
        if (b) {
            test_label3:
                for (; ; ) {}
        }
        assert false;
        assert true : "false";
        f9 = (Object) f9;
        f9.equals(vstring = "");
        for (int i = 0; i < 12; i++) ;
        vint = vboolean ? (vint = 1) : (vint = 0);
        varray[vint] = 0;
    }
    public @interface innerAnnotation {
    }
}
