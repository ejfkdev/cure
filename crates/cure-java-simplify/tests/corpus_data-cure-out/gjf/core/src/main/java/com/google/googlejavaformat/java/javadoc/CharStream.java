package com.google.googlejavaformat.java.javadoc;

import static com.google.common.base.Preconditions.checkNotNull;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

final class CharStream {
    private final String input;
    private int position;
    private int tokenEnd = -1;
    CharStream(String input) {
        this.input = checkNotNull(input);
    }
    boolean tryConsume(String expected) {
        if (!input.startsWith(expected, position)) {
            return false;
        }
        tokenEnd = position + expected.length();
        return true;
    }
    boolean tryConsumeRegex(Pattern pattern) {
        Matcher matcher = pattern.matcher(input).region(position, input.length());
        if (!matcher.lookingAt()) {
            return false;
        }
        tokenEnd = matcher.end();
        return true;
    }
    String readAndResetRecorded() {
        String result = input.substring(position, tokenEnd);
        position = tokenEnd;
        tokenEnd = -1;
        return result;
    }
    boolean isExhausted() {
        return position == input.length();
    }
    int position() {
        return position;
    }
}
