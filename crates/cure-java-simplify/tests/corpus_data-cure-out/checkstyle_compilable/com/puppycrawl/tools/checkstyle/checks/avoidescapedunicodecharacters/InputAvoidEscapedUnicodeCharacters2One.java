package com.puppycrawl.tools.checkstyle.checks.avoidescapedunicodecharacters;

import java.util.concurrent.TimeUnit;

public class InputAvoidEscapedUnicodeCharacters2One {
    private String unitAbbrev2 = "μs";
    private String unitAbbrev3 = "μs";
    private String unitAbbrev4 = "μs";
    public Object fooString() {
        return "﻿" + null;
    }
    public Object fooChar() {
        return 65279;
    }
    public void multiplyString() {
        String allCharactersEscaped = "μμ";
    }
    private static String abbreviate(TimeUnit unit) {
        switch (unit) {
            case NANOSECONDS:
                return "ns";
            case MICROSECONDS:
                return "μs";
            case MILLISECONDS:
                return "ms";
            case SECONDS:
                return "s";
            case MINUTES:
                return "min";
            case HOURS:
                return "h";
            case DAYS:
                return "d";
            default:
                throw new AssertionError();
        }
    }
    static final String WHITESPACE_TABLE = " 　\r   　\\ \u000B　   　 \t     \u000C " + "　 　　 \n 　";
    public boolean matches(char c) {
        switch (c) {
            case '\t':
            case '\n':
            case '\u0000':
            case '\u000C':
            case '\r':
            case ' ':
            case '':
            case ' ':
            case ' ':
            case ' ':
            case ' ':
            case '　':
                return true;
            case ' ':
                return false;
            default:
                return c >= ' ' && c <= ' ';
        }
    }
}
