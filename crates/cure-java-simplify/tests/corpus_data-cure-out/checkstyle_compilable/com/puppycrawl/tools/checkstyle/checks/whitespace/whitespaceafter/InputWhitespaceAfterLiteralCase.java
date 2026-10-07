package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespaceafter;

public class InputWhitespaceAfterLiteralCase {
    public static void main(String... args) {
        switch (args[0]) {
            case "123":
                return;
            case "1":
                return;
            default:
                return;
        }
    }
}
