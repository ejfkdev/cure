package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

class InputLeftCurlyTestDefault3Initializer {
    static {
        int x = 1;
    }
}

class ClassWithStaticInitializers {
    static {}
    static {}
    static class Inner {
        static {
            int i = 1;
        }
    }
}
