package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionPatternsInIfStatement {
    void m1(Object o) {
        if (o instanceof String s && s.length() > 6) {}
    }
}
