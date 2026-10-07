package com.puppycrawl.tools.checkstyle.checks.avoidescapedunicodecharacters;

public class InputAvoidEscapedUnicodeCharactersTextBlocksAllowByComment {
    public void multiplyString1() {
        String allCharactersEscaped = "μμ";
    }
    public void multiplyString2() {
        String allCharactersEscaped = """
                μμ""";
    }
}
