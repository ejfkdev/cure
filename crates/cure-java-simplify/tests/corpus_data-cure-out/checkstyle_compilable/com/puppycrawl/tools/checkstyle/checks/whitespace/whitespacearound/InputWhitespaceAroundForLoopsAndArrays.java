package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

class InputWhitespaceAroundForLoopsAndArrays {
    void forIterator() {
        for (int i = 0; i++ < 5; ) {}
        int i = 0;
        for (; i < 5; i++) {}
        for (int anInt : getSomeInts()) {}
    }
    int[] getSomeInts() {
        return null;
    }
    public void myMethod() {
        new Thread() {
            public void run() {
            }
        }.start();
    }
    public void foo(java.util.List<? extends String[]> bar, Comparable<? super Object[]> baz) {}
    public void mySuperMethod() {
        new Runnable() {
                public void run() {
                }
            }.run();
    }
    public void testNullSemi() {}
    public void register(Object obj) {}
    public void doSomething(String[] args) {
        register(boolean[].class);
        register(args);
    }
    public void parentheses() {
        testNullSemi();
    }
    public static void testNoWhitespaceBeforeEllipses(String... args) {}
    public String test() {
        return "00000";
    }
}
