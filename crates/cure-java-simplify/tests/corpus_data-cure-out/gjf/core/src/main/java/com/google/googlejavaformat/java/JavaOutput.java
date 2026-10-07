package com.google.googlejavaformat.java;

import static java.lang.Math.min;
import static java.util.Comparator.comparing;
import com.google.common.base.CharMatcher;
import com.google.common.base.MoreObjects;
import com.google.common.collect.DiscreteDomain;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import com.google.common.collect.RangeSet;
import com.google.common.collect.TreeRangeSet;
import com.google.googlejavaformat.CommentsHelper;
import com.google.googlejavaformat.Input;
import com.google.googlejavaformat.Input.Token;
import com.google.googlejavaformat.Newlines;
import com.google.googlejavaformat.OpsBuilder.BlankLineWanted;
import com.google.googlejavaformat.Output;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.IntFunction;

public final class JavaOutput extends Output {
    private final String lineSeparator;
    private final Input javaInput;
    private final CommentsHelper commentsHelper;
    private final IntFunction<String> indentFunction;
    private final Map<Integer, BlankLineWanted> blankLines = new HashMap<>();
    private final RangeSet<Integer> partialFormatRanges = TreeRangeSet.create();
    private final List<String> mutableLines = new ArrayList<>();
    private final int kN;
    private int iLine = 0;
    private int lastK = -1;
    private int newlinesPending = 0;
    private StringBuilder lineBuilder = new StringBuilder();
    private StringBuilder spacesPending = new StringBuilder();
    private static final int MAX_CACHED_SPACES = 100;
    private static final String[] spaces = new String[MAX_CACHED_SPACES + 1];
    static String spaces(int indent) {
        if (indent <= 0) {
            return "";
        }
        if (indent <= MAX_CACHED_SPACES) {
            String result = spaces[indent];
            if (result == null) {
                result = spaces[indent] = " ".repeat(indent);
            }
            return result;
        }
        return " ".repeat(indent);
    }
    public JavaOutput(String lineSeparator, Input javaInput, CommentsHelper commentsHelper) {
        this(lineSeparator, javaInput, commentsHelper, JavaOutput::spaces);
    }
    public JavaOutput(String lineSeparator, Input javaInput, CommentsHelper commentsHelper, IntFunction<String> indentFunction) {
        this.lineSeparator = lineSeparator;
        this.javaInput = javaInput;
        this.commentsHelper = commentsHelper;
        this.indentFunction = indentFunction;
        kN = javaInput.getkN();
    }
    @Override
  public void blankLine(int k, BlankLineWanted wanted) {
        if (blankLines.containsKey(k)) {
            blankLines.put(k, blankLines.get(k).merge(wanted));
        } else {
            blankLines.put(k, wanted);
        }
    }
    @Override
  public void markForPartialFormat(Token start, Token end) {
        int lo = JavaOutput.startTok(start).getIndex();
        int hi = JavaOutput.endTok(end).getIndex();
        partialFormatRanges.add(Range.closed(lo, hi));
    }
    @Override
  public void append(String text, Range<Integer> range) {
        if (!range.isEmpty()) {
            boolean sawNewlines = false;
            int iN = javaInput.getLineCount();
            while (iLine < iN && (javaInput.getRanges(iLine).isEmpty() || javaInput.getRanges(iLine).upperEndpoint() <= range.lowerEndpoint())) {
                if (javaInput.getRanges(iLine).isEmpty()) {
                    sawNewlines = true;
                }
                ++iLine;
            }
            BlankLineWanted wanted = blankLines.getOrDefault(lastK, BlankLineWanted.NO);
            if (sawNewlines && isComment(text) || wanted.wanted().orElse(sawNewlines)) {
                ++newlinesPending;
            }
        }
        if (Newlines.isNewline(text)) {
            if (newlinesPending == 0) {
                ++newlinesPending;
            }
            spacesPending = new StringBuilder();
        } else {
            boolean rangesSet = false;
            int textN = text.length();
            for (int i = 0; i < textN; i++) {
                char c = text.charAt(i);
                switch (c) {
                    case ' ':
                        spacesPending.append(' ');
                        break;
                    case '\t':
                        spacesPending.append('\t');
                        break;
                    case '\r':
                        if (i + 1 < text.length() && text.charAt(i + 1) == '\n') {
                            i++;
                        }
                    case '\n':
                        spacesPending = new StringBuilder();
                        ++newlinesPending;
                        break;
                    default:
                        while (newlinesPending > 0) {
                            if (!mutableLines.isEmpty() || lineBuilder.length() > 0) {
                                mutableLines.add(lineBuilder.toString());
                            }
                            lineBuilder = new StringBuilder();
                            rangesSet = false;
                            --newlinesPending;
                        }
                        if (spacesPending.length() > 0) {
                            lineBuilder.append(spacesPending);
                            spacesPending = new StringBuilder();
                        }
                        lineBuilder.append(c);
                        if (!range.isEmpty()) {
                            if (!rangesSet) {
                                while (ranges.size() <= mutableLines.size()) {
                                    ranges.add(EMPTY_RANGE);
                                }
                                ranges.set(mutableLines.size(), union(ranges.get(mutableLines.size()), range));
                                rangesSet = true;
                            }
                        }
                }
            }
        }
        if (!range.isEmpty()) {
            lastK = range.upperEndpoint();
        }
    }
    @Override
  public void indent(int indent) {
        spacesPending.append(indentFunction.apply(indent));
    }
    public void flush() {
        String lastLine = lineBuilder.toString();
        if (!CharMatcher.whitespace().matchesAllOf(lastLine)) {
            mutableLines.add(lastLine);
        }
        int jN = mutableLines.size();
        Range<Integer> eofRange = Range.closedOpen(kN, kN + 1);
        while (ranges.size() < jN) {
            ranges.add(EMPTY_RANGE);
        }
        ranges.add(eofRange);
        setLines(ImmutableList.copyOf(mutableLines));
    }
    @Override
  public CommentsHelper getCommentsHelper() {
        return commentsHelper;
    }
    public ImmutableList<Replacement> getFormatReplacements(RangeSet<Integer> iRangeSet0) {
        ImmutableList.Builder<Replacement> result = ImmutableList.builder();
        Map<Integer, Range<Integer>> kToJ = JavaOutput.makeKToIJ(this);
        RangeSet<Integer> breakableRanges = TreeRangeSet.create();
        RangeSet<Integer> iRangeSet = iRangeSet0.subRangeSet(Range.closed(0, javaInput.getkN()));
        for (Range<Integer> iRange : iRangeSet.asRanges()) {
            Range<Integer> range = expandToBreakableRegions(iRange.canonical(DiscreteDomain.integers()));
            if (range.equals(EMPTY_RANGE)) {
                continue;
            }
            breakableRanges.add(range);
        }
        for (Range<Integer> range : breakableRanges.asRanges()) {
            Input.Tok startTok = startTok(javaInput.getToken(range.lowerEndpoint()));
            Input.Tok endTok = endTok(javaInput.getToken(range.upperEndpoint() - 1));
            StringBuilder replacement = new StringBuilder();
            int replaceFrom = startTok.getPosition();
            while (replaceFrom > 0) {
                char previous = javaInput.getText().charAt(replaceFrom - 1);
                if (!CharMatcher.whitespace().matches(previous)) {
                    break;
                }
                replaceFrom--;
            }
            int i = kToJ.get(startTok.getIndex()).lowerEndpoint();
            while (i > 0 && getLine(i - 1).isEmpty()) {
                i--;
            }
            for (; i < kToJ.get(endTok.getIndex()).upperEndpoint(); i++) {
                if (i < getLineCount()) {
                    if (i > 0) {
                        replacement.append(lineSeparator);
                    }
                    replacement.append(getLine(i));
                }
            }
            int replaceTo = min(endTok.getPosition() + endTok.length(), javaInput.getText().length());
            if (endTok.getIndex() == javaInput.getkN() - 1) {
                replaceTo = javaInput.getText().length();
            }
            int newline = -1;
            while (replaceTo < javaInput.getText().length()) {
                char next = javaInput.getText().charAt(replaceTo);
                if (!CharMatcher.whitespace().matches(next)) {
                    break;
                }
                int newlineLength = Newlines.hasNewlineAt(javaInput.getText(), replaceTo);
                if (newlineLength != -1) {
                    newline = replaceTo;
                    replaceTo += newlineLength;
                } else {
                    replaceTo++;
                }
            }
            if (newline != -1) {
                replaceTo = newline;
            }
            if (newline == -1) {
                replacement.append(lineSeparator);
            }
            for (; i < getLineCount(); i++) {
                String after = getLine(i);
                int idx = CharMatcher.whitespace().negate().indexIn(after);
                if (idx == -1) {
                    replacement.append(lineSeparator);
                } else {
                    if (newline == -1) {
                        replacement.append(after, 0, idx);
                    }
                    break;
                }
            }
            result.add(Replacement.create(replaceFrom, replaceTo, replacement.toString()));
        }
        return result.build();
    }
    private Range<Integer> expandToBreakableRegions(Range<Integer> iRange) {
        int loTok = iRange.lowerEndpoint();
        int hiTok = iRange.upperEndpoint() - 1;
        if (!partialFormatRanges.contains(loTok) || !partialFormatRanges.contains(hiTok)) {
            return EMPTY_RANGE;
        }
        loTok = partialFormatRanges.rangeContaining(loTok).lowerEndpoint();
        hiTok = partialFormatRanges.rangeContaining(hiTok).upperEndpoint();
        return Range.closedOpen(loTok, hiTok + 1);
    }
    public static String applyReplacements(String input, List<Replacement> replacements) {
        replacements = new ArrayList<>(replacements);
        replacements.sort(comparing((Replacement r) -> r.replaceRange().lowerEndpoint()).reversed());
        StringBuilder writer = new StringBuilder(input);
        for (Replacement replacement : replacements) {
            writer.replace(replacement.replaceRange().lowerEndpoint(), replacement.replaceRange().upperEndpoint(), replacement.replacementString());
        }
        return writer.toString();
    }
    public static Input.Tok startTok(Token token) {
        for (Input.Tok tok : token.getToksBefore()) {
            if (tok.getIndex() >= 0) {
                return tok;
            }
        }
        return token.getTok();
    }
    public static Input.Tok endTok(Token token) {
        for (int i = token.getToksAfter().size() - 1; i >= 0; i--) {
            Input.Tok tok = token.getToksAfter().get(i);
            if (tok.getIndex() >= 0) {
                return tok;
            }
        }
        return token.getTok();
    }
    private boolean isComment(String text) {
        return text.startsWith("//") || text.startsWith("/*");
    }
    private static Range<Integer> union(Range<Integer> x, Range<Integer> y) {
        return x.isEmpty() ? y : y.isEmpty() ? x : x.span(y).canonical(DiscreteDomain.integers());
    }
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("iLine", iLine).add("lastK", lastK).add("spacesPending", spacesPending.toString().replace("\t", "\\t")).add("newlinesPending", newlinesPending).add("blankLines", blankLines).add("super", super.toString()).toString();
    }
}
