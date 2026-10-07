package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.io.Serializable;

public class InputAntlr4AstRegressionBadOverride {
    public void doFoo() {}
    public void doFoo2() {}
}

interface IFoo2 {
    void doFoo();
}

interface IBar2 extends IFoo2 {
    public void doFoo();
}

class MoreJunk2 extends InputAntlr4AstRegressionBadOverride {
    public void doFoo() {}
    public void doFoo2() {}
    class EvenMoreJunk extends MoreJunk2 implements Serializable {
        public void doFoo() {}
        public void doFoo2() {}
    }
}

enum Football2 implements IFoo2, IBar2 {
    Detroit_Lions;
    public void doFoo() {}
}
