package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

import java.io.IOException;

public class InputJavadocMethodTags1Two {
    void method14(int aOne, int aTwo, int aThree, int aFour, int aFive) {}
    void method15(int aOne) {}
    void method15() throws java.io.IOException {}
    static {
        int x = 1;
    }
    {
        int z = 2;
    }
    private static final int ON_SECOND_LINE = 2;
    public String toString() {
        return super.toString();
    }
    static final int serialVersionUID = 666;
    void method16(int aOne) {}
    void method17() throws IllegalMonitorStateException {}
    void method18() throws IOException {
        throw new IOException("to make compiler happy");
    }
    void method19() throws java.io.IOException {
        throw new IOException("to make compiler happy");
    }
}
