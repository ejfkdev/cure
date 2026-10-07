package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionPatternsInSwitch {
    void m1(Object o) {
        switch (o) {
            case String s when s.length() > 4:
                break;
            case String s:
                break;
            case null:
            case default:
                throw new UnsupportedOperationException("not supported!");
        }
    }
}
