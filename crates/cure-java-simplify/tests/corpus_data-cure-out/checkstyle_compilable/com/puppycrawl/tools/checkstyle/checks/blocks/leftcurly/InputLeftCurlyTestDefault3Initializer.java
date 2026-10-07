package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

class InputLeftCurlyTestDefault3Initializer {
    static {}
}

class ClassWithStaticInitializers {
    static {}
    static {}
    static class Inner {
        static {}
    }
}
