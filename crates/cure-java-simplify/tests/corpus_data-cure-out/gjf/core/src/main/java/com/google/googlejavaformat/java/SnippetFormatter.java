package com.google.googlejavaformat.java;

import static com.google.common.collect.ImmutableList.toImmutableList;
import com.google.common.base.CharMatcher;
import com.google.common.base.Preconditions;
import com.google.common.collect.DiscreteDomain;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import com.google.common.collect.RangeSet;
import com.google.common.collect.TreeRangeSet;
import java.util.ArrayList;
import java.util.List;

public class SnippetFormatter {
    public enum SnippetKind {
        COMPILATION_UNIT, CLASS_BODY_DECLARATIONS, STATEMENTS, EXPRESSION
    }
    private class SnippetWrapper {
        int offset;
        final StringBuilder contents = new StringBuilder();
        public SnippetWrapper append(String str) {
            contents.append(str);
            return this;
        }
        public SnippetWrapper appendSource(String source) {
            this.offset = contents.length();
            contents.append(source);
            return this;
        }
        public void closeBraces(int initialIndent) {
            for (int i = initialIndent; --i >= 0; ) {
                contents.append("\n").append(createIndentationString(i)).append("}");
            }
        }
    }
    private static final int INDENTATION_SIZE = 2;
    private final Formatter formatter;
    private static final CharMatcher NOT_WHITESPACE = CharMatcher.whitespace().negate();
    public SnippetFormatter() {
        this(JavaFormatterOptions.defaultOptions());
    }
    public SnippetFormatter(JavaFormatterOptions formatterOptions) {
        formatter = new Formatter(formatterOptions);
    }
    private static String createIndentationString(int indentationLevel) {
        Preconditions.checkArgument(indentationLevel >= 0, "Indentation level cannot be less than zero. Given: %s", indentationLevel);
        return " ".repeat(indentationLevel * INDENTATION_SIZE);
    }
    private static Range<Integer> offsetRange(Range<Integer> range, int offset) {
        range = range.canonical(DiscreteDomain.integers());
        return Range.closedOpen(range.lowerEndpoint() + offset, range.upperEndpoint() + offset);
    }
    private static List<Range<Integer>> offsetRanges(List<Range<Integer>> ranges, int offset) {
        List<Range<Integer>> result = new ArrayList<>();
        for (Range<Integer> range : ranges) {
            result.add(offsetRange(range, offset));
        }
        return result;
    }
    public ImmutableList<Replacement> format(SnippetKind kind, String source, List<Range<Integer>> ranges, int initialIndent, boolean includeComments) throws FormatterException {
        RangeSet<Integer> rangeSet = TreeRangeSet.create();
        for (Range<Integer> range : ranges) {
            rangeSet.add(range);
        }
        if (includeComments) {
            if (kind != SnippetKind.COMPILATION_UNIT) {
                throw new IllegalArgumentException("comment formatting is only supported for compilation units");
            }
            return formatter.getFormatReplacements(source, ranges);
        }
        SnippetWrapper wrapper = snippetWrapper(kind, source, initialIndent);
        ranges = offsetRanges(ranges, wrapper.offset);
        String replacement = formatter.formatSource(wrapper.contents.toString(), ranges);
        replacement = replacement.substring(wrapper.offset, replacement.length() - (wrapper.contents.length() - wrapper.offset - source.length()));
        return toReplacements(source, replacement).stream().filter((r) -> rangeSet.encloses(r.replaceRange())).collect(toImmutableList());
    }
    private static List<Replacement> toReplacements(String source, String replacement) {
        if (!NOT_WHITESPACE.retainFrom(source).equals(NOT_WHITESPACE.retainFrom(replacement))) {
            throw new IllegalArgumentException("source = \"" + source + "\", replacement = \"" + replacement + "\"");
        }
        List<Replacement> replacements = new ArrayList<>();
        int i = NOT_WHITESPACE.indexIn(source);
        int j = NOT_WHITESPACE.indexIn(replacement);
        if (i != 0 || j != 0) {
            replacements.add(Replacement.create(0, i, replacement.substring(0, j)));
        }
        while (i != -1 && j != -1) {
            int i2 = NOT_WHITESPACE.indexIn(source, i + 1);
            int j2 = NOT_WHITESPACE.indexIn(replacement, j + 1);
            if (i2 == -1 || j2 == -1) {
                break;
            }
            if (i2 - i != j2 - j || !source.substring(i + 1, i2).equals(replacement.substring(j + 1, j2))) {
                replacements.add(Replacement.create(i + 1, i2, replacement.substring(j + 1, j2)));
            }
            i = i2;
            j = j2;
        }
        return replacements;
    }
    private SnippetWrapper snippetWrapper(SnippetKind kind, String source, int initialIndent) {
        return switch (kind) {
            case COMPILATION_UNIT, CLASS_BODY_DECLARATIONS -> {
                SnippetWrapper wrapper = new SnippetWrapper();
                for (int i = 1; i <= initialIndent; i++) {
                    wrapper.append("class Dummy {\n").append(createIndentationString(i));
                }
                wrapper.appendSource(source);
                wrapper.closeBraces(initialIndent);
                yield wrapper;
            }
            case STATEMENTS -> {
                SnippetWrapper wrapper = new SnippetWrapper();
                wrapper.append("class Dummy {\n").append(createIndentationString(1));
                for (int i = 2; i <= initialIndent; i++) {
                    wrapper.append("{\n").append(createIndentationString(i));
                }
                wrapper.appendSource(source);
                wrapper.closeBraces(initialIndent);
                yield wrapper;
            }
            case EXPRESSION -> {
                SnippetWrapper wrapper = new SnippetWrapper();
                wrapper.append("class Dummy {\n").append(createIndentationString(1));
                for (int i = 2; i <= initialIndent; i++) {
                    wrapper.append("{\n").append(createIndentationString(i));
                }
                wrapper.append("Object o = ");
                wrapper.appendSource(source);
                wrapper.append(";");
                wrapper.closeBraces(initialIndent);
                yield wrapper;
            }
        };
    }
}
