package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

public class InputRightCurlyTestWithComment {
    void m1(int mode) {
        switch (mode) {
            default:
                int x = 0;
        }
    }
    void m2() {}
    void method() {}
    void loop() {
        while (true) {
            System.out.println("Checkstyle");
        }
    }
}
