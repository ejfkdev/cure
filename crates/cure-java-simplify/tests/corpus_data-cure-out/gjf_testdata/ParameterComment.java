package com.google.googlejavaformat.java.test;

class Q {
    static void f(int a) {
        f(1);
        f(1);
    }
    static void g(int a, int b, int c, int d, int e, int f, int g, int h, int i, int j, int k) {
        g(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1);
        g(1, 1, 1, 1, 1, 1, 1, 1, 1);
    }
    static void h(Object... xs) {
        h(null);
    }
}
