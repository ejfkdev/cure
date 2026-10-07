package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionPatternsInWhile {
    void m1(Object o) {
        while (o instanceof String s && s.length() > 4) {}
        while (o instanceof String s) {}
        do {} while (o instanceof String s && s.length() > 4);
        do {} while (o instanceof String s);
    }
}
