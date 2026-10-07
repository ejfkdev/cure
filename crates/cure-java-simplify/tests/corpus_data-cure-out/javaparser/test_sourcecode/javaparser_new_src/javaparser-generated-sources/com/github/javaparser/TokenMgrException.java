package com.github.javaparser;

public class TokenMgrException extends RuntimeException {
    private static final long serialVersionUID = 1L;
    public static final int LEXICAL_ERROR = 0;
    public static final int STATIC_LEXER_ERROR = 1;
    public static final int INVALID_LEXICAL_STATE = 2;
    public static final int LOOP_DETECTED = 3;
    int errorCode;
    protected static final String addEscapes(String str) {
        StringBuffer retval = new StringBuffer();
        char ch;
        for (int i = 0; i < str.length(); i++) {
            switch (str.charAt(i)) {
                case '\u0008':
                    retval.append("\\b");
                    continue;
                case '\t':
                    retval.append("\\t");
                    continue;
                case '\n':
                    retval.append("\\n");
                    continue;
                case '\u000C':
                    retval.append("\\f");
                    continue;
                case '\r':
                    retval.append("\\r");
                    continue;
                case '"':
                    retval.append("\\\"");
                    continue;
                case '\'':
                    retval.append("\\'");
                    continue;
                case '\\':
                    retval.append("\\\\");
                    continue;
                default:
                    if ((ch = str.charAt(i)) < 0x20 || ch > 0x7e) {
                        String s = "0000" + Integer.toString(ch, 16);
                        retval.append("\\u" + s.substring(s.length() - 4, s.length()));
                    } else {
                        retval.append(ch);
                    }
                    continue;
            }
        }
        return retval.toString();
    }
    protected static String LexicalErr(boolean EOFSeen, int lexState, int errorLine, int errorColumn, String errorAfter, int curChar) {
        char curChar1 = (char) curChar;
        return "Lexical error at line " + errorLine + ", column " + errorColumn + ".  Encountered: " + (EOFSeen ? "<EOF> " : "\"" + addEscapes(String.valueOf(curChar1)) + "\" (" + curChar + "), ") + "after : \"" + addEscapes(errorAfter) + "\"";
    }
    public String getMessage() {
        return super.getMessage();
    }
    public TokenMgrException() {}
    public TokenMgrException(String message, int reason) {
        super(message);
        errorCode = reason;
    }
    public TokenMgrException(boolean EOFSeen, int lexState, int errorLine, int errorColumn, String errorAfter, int curChar, int reason) {
        this(LexicalErr(EOFSeen, lexState, errorLine, errorColumn, errorAfter, curChar), reason);
    }
}
