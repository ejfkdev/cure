package com.puppycrawl.tools.checkstyle.checks.whitespace.operatorwrap;

import java.util.Arrays;

class InputOperatorWrap2 {
    void test() {
        Arrays.sort(null, String::compareToIgnoreCase);
        Arrays.sort(null, String::compareToIgnoreCase);
        Arrays.sort(null, String::compareToIgnoreCase);
    }
    void testAssignment() {
        int y = 0;
    }
    <
        T extends Comparable &
        java.io.Serializable
    > void testGenerics1() {
        Comparable<
            String
            > c = "";
    }
}

class badCase22<T extends Foo2 &
    Bar2> {
}

class goodCase2<T extends Foo2 & Bar2> {
}

class Switch2 {
    public void test(int i, int j) {
        switch (j) {
            case 7:
                return;
        }
        switch (i) {
            case 1:
                break;
            default:

        }
        for (int k : new int[] {1, 2, 3}) {}
    }
}

interface Foo2 {
}

interface Bar2 {
}
