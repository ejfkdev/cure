package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionPatternsInTernary {
    void m1(Object o) {
        int y = o instanceof String s && s.length() > 4 ? 2 : 3;
        int z = o instanceof String s ? 1 : 2;
    }
}
