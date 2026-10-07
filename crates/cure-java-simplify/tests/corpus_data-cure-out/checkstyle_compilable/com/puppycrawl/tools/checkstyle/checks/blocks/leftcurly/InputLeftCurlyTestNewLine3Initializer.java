package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

class InputLeftCurlyTestNewLine3Initializer {
    static {
        int x = 1;
    }
}

class ClassWithStaticInitializersTestNewLine3 {
    static {}
    static {}
    static class Inner {
        static {
            int i = 1;
        }
    }
}
