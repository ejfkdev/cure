package com.puppycrawl.tools.checkstyle.grammar.java21;

import java.util.List;

public class InputTextBlockConsecutiveEscapes {
    String s = """
            μ\t
            μ\s
            μ\s not all escaped chars
            μ\n not all escaped chars
            \s\s\s\n
            μ\s
            \s\s\s\n not all escaped chars
            μ\s not all escaped chars
            μ\n not all escaped chars
            lμ\n
            \n       μ\s
            μ\
            \sμ\
            """;
    final List<TestCase> testCases = List.of(new TestCase("""
                    {@snippet :
                    hello there //   @highlight   regex ="\t**"
                    }""", """
                    error: snippet markup: invalid regex
                    hello there //   @highlight   regex ="\t**"
                                                          \t ^
                    """), new TestCase("""
                    {@snippet :
                    hello there //   @highlight   regex ="\\t**"
                    }""", """
                    error: snippet markup: invalid regex
                    hello there //   @highlight   regex ="\\t**"
                                                             ^
                    """), new TestCase("""
                    {@snippet :
                    hello there // @highlight regex="\\.\\*\\+\\E"
                    }""", """
                    error: snippet markup: invalid regex
                    hello there // @highlight regex="\\.\\*\\+\\E"
                                                     \s\s\s\s   ^
                    """), new TestCase("""
                    {@snippet :
                    hello there //   @highlight  type="italics" regex ="  ["
                    }""", """
                    error: snippet markup: invalid regex
                    hello there //   @highlight  type="italics" regex ="  ["
                                                                          ^
                    """));
    private record TestCase(String s1, String s2) {
    }
}
