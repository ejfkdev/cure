package com.puppycrawl.tools.checkstyle.checks.blocks.googlerightcurly;

public class InputGoogleRightCurlyConciseBlockNotAloneOnLine {
    static {}
    int b;
    void method() {}
    void method2() {}
    void method3() {
        int a = 2;
        if (a == 2) {
            a++;
        } else if (a == 3) {
            a++;
        } else {
            a = 1;
        }
    }
    static class Inner {
    }
    static class Inner2 {
    }
}

class AnotherClass {
}
