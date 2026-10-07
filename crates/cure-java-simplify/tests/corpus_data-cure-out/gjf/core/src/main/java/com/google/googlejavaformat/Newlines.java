package com.google.googlejavaformat;

import com.google.common.base.CharMatcher;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Iterators;
import java.util.Iterator;
import java.util.NoSuchElementException;

public class Newlines {
    public static int count(String input) {
        return Iterators.size(lineOffsetIterator(input)) - 1;
    }
    public static int firstBreak(String input) {
        Iterator<Integer> it = lineOffsetIterator(input);
        it.next();
        return it.hasNext() ? it.next() : -1;
    }
    private static final ImmutableSet<String> BREAKS = ImmutableSet.of("\r\n", "\n", "\r");
    public static boolean isNewline(String input) {
        return BREAKS.contains(input);
    }
    public static int hasNewlineAt(String input, int idx) {
        for (String b : BREAKS) {
            if (input.startsWith(b, idx)) {
                return b.length();
            }
        }
        return -1;
    }
    public static String getLineEnding(String input) {
        for (String b : BREAKS) {
            if (input.endsWith(b)) {
                return b;
            }
        }
        return null;
    }
    public static String guessLineSeparator(String text) {
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            switch (c) {
                case '\r' -> {
                    return i + 1 < text.length() && text.charAt(i + 1) == '\n' ? "\r\n" : "\r";
                }
                case '\n' -> {
                    return "\n";
                }
                default -> {}
            }
        }
        return "\n";
    }
    public static boolean containsBreaks(String text) {
        return CharMatcher.anyOf("\n\r").matchesAnyOf(text);
    }
    public static Iterator<Integer> lineOffsetIterator(String input) {
        return new LineOffsetIterator(input);
    }
    public static Iterator<String> lineIterator(String input) {
        return new LineIterator(input);
    }
    private static class LineOffsetIterator implements Iterator<Integer> {
        private int curr = 0;
        private int idx = 0;
        private final String input;
        private LineOffsetIterator(String input) {
            this.input = input;
        }
        @Override
    public boolean hasNext() {
            return curr != -1;
        }
        @Override
    public Integer next() {
            if (curr == -1) {
                throw new NoSuchElementException();
            }
            advance();
            return curr;
        }
        private void advance() {
            for (; idx < input.length(); idx++) {
                char c = input.charAt(idx);
                switch (c) {
                    case '\r':
                        if (idx + 1 < input.length() && input.charAt(idx + 1) == '\n') {
                            idx++;
                        }
                    case '\n':
                        idx++;
                        curr = idx;
                        return;
                    default:
                        break;
                }
            }
            curr = -1;
        }
        @Override
    public void remove() {
            throw new UnsupportedOperationException("remove");
        }
    }
    private static class LineIterator implements Iterator<String> {
        int idx;
        String curr;
        private final String input;
        private final Iterator<Integer> indices;
        private LineIterator(String input) {
            this.input = input;
            this.indices = lineOffsetIterator(input);
            idx = indices.next();
        }
        private void advance() {
            int last = idx;
            if (indices.hasNext()) {
                idx = indices.next();
            } else if (hasNext()) {
                idx = input.length();
            } else {
                throw new NoSuchElementException();
            }
            curr = input.substring(last, idx);
        }
        @Override
    public boolean hasNext() {
            return idx < input.length();
        }
        @Override
    public String next() {
            advance();
            return curr;
        }
        @Override
    public void remove() {
            throw new UnsupportedOperationException("remove");
        }
    }
}
