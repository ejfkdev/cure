package com.puppycrawl.tools.checkstyle.grammar.comments;

public class InputFullOfBlockComments {
    public/*20*/ static String main(String[] args) {
        String line = "/*I'm NOT comment*/blabla";
        String.CASE_INSENSITIVE_ORDER.equals(line);
        for (Integer i : null) {}
        return line;
    }
}
