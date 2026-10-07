package com.google.googlejavaformat.java.test;

class B {
    int x;
    private int y;
    public int z;
    void f() {
        LABEL:
            while (true) {
                break LABEL;
            }
    }
}
