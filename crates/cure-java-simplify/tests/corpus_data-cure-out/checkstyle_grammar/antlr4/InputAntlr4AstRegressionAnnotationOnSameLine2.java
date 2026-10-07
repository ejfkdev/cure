package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionAnnotationOnSameLine2 {
    public int i;
    public void foo() {}
    @Deprecated
        public void foo2() {}
    @Deprecated
        public void foo3() {}
    @Deprecated
        public void foo4() {}
    @Deprecated
        public void foo5() {}
    void local(@Deprecated String s) {}
    void local2(String s) {}
    void local3(@Deprecated String s) {}
    void dontUse() {}
    @Deprecated void dontUse2() {}
    int[] dontUse3() {
        return null;
    }
    <T> T dontUse4() {
        return null;
    }
    java.lang.String dontUse5() {
        return null;
    }
}
