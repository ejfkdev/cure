package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocvariable;

import java.io.IOException;

class InputJavadocVariableTagsMethods2 {
    void method14(int aOne) {}
    void method14() throws java.io.IOException {}
    static {
        int x = 1;
    }
    {
        int z = 2;
    }
    private static final int ON_SECOND_LINE = 2;
    void method15() throws java.io.IOException {}
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
