package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionPatternsInFor {
    void m1(Object o) {
        for (int i = 0; o instanceof Integer myInt && myInt > 5; ) {}
        for (int i = 0; o instanceof Integer myInt; ) {}
    }
}
