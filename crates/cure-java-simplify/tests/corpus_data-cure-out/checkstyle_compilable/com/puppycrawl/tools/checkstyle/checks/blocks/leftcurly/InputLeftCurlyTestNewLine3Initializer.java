package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

class InputLeftCurlyTestNewLine3Initializer {
    static {}
}

class ClassWithStaticInitializersTestNewLine3 {
    static {}
    static {}
    static class Inner {
        static {}
    }
}
