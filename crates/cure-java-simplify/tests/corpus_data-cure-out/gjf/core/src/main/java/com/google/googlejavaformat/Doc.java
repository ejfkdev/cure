package com.google.googlejavaformat;

import static com.google.common.collect.Iterables.getLast;
import static com.google.googlejavaformat.CommentsHelper.reformatParameterComment;
import static java.lang.Math.max;
import com.google.common.base.MoreObjects;
import com.google.common.base.Supplier;
import com.google.common.base.Suppliers;
import com.google.common.collect.DiscreteDomain;
import com.google.common.collect.Iterators;
import com.google.common.collect.Range;
import com.google.googlejavaformat.Output.BreakTag;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public abstract class Doc {
    public enum FillMode {
        UNIFIED, INDEPENDENT, FORCED
    }
    public static final int MAX_LINE_WIDTH = 1000;
    public record State(int lastIndent, int indent, int column, boolean mustBreak) {
        public State(int indent0, int column0) {
            this(indent0, indent0, column0, false);
        }
        State withColumn(int column) {
            return new State(lastIndent, indent, column, mustBreak);
        }
        State withMustBreak(boolean mustBreak) {
            return new State(lastIndent, indent, column, mustBreak);
        }
    }
    private static final Range<Integer> EMPTY_RANGE = Range.closedOpen(-1, -1);
    private static final DiscreteDomain<Integer> INTEGERS = DiscreteDomain.integers();
    private final Supplier<Integer> width = Suppliers.memoize(this::computeWidth);
    private final Supplier<String> flat = Suppliers.memoize(this::computeFlat);
    private final Supplier<Range<Integer>> range = Suppliers.memoize(this::computeRange);
    final int getWidth() {
        return width.get();
    }
    final String getFlat() {
        return flat.get();
    }
    final Range<Integer> range() {
        return range.get();
    }
    abstract int computeWidth();
    abstract String computeFlat();
    abstract Range<Integer> computeRange();
    public abstract State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state);
    public abstract void write(Output output);
    static final class Level extends Doc {
        private final Indent plusIndent;
        private final List<Doc> docs = new ArrayList<>();
        private Level(Indent plusIndent) {
            this.plusIndent = plusIndent;
        }
        static Level make(Indent plusIndent) {
            return new Level(plusIndent);
        }
        void add(Doc doc) {
            docs.add(doc);
        }
        @Override int computeWidth() {
            return getWidth(docs);
        }
        @Override String computeFlat() {
            StringBuilder builder = new StringBuilder();
            for (Doc doc : docs) {
                builder.append(doc.getFlat());
            }
            return builder.toString();
        }
        @Override Range<Integer> computeRange() {
            Range<Integer> docRange = EMPTY_RANGE;
            for (Doc doc : docs) {
                docRange = union(docRange, doc.range());
            }
            return docRange;
        }
        boolean oneLine = false;
        List<List<Doc>> splits = new ArrayList<>();
        List<Break> breaks = new ArrayList<>();
        @Override
    public State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state) {
            int thisWidth = getWidth();
            if (state.column + thisWidth <= maxWidth) {
                oneLine = true;
                return state.withColumn(state.column + thisWidth);
            }
            State broken = computeBroken(commentsHelper, maxWidth, new State(state.indent + plusIndent.eval(), state.column));
            return state.withColumn(broken.column);
        }
        private static void splitByBreaks(List<Doc> docs, List<List<Doc>> splits, List<Break> breaks) {
            splits.clear();
            breaks.clear();
            splits.add(new ArrayList<>());
            for (Doc doc : docs) {
                if (doc instanceof Break b) {
                    breaks.add(b);
                    splits.add(new ArrayList<>());
                } else {
                    getLast(splits).add(doc);
                }
            }
        }
        private State computeBroken(CommentsHelper commentsHelper, int maxWidth, State state) {
            splitByBreaks(docs, splits, breaks);
            state = computeBreakAndSplit(commentsHelper, maxWidth, state, Optional.empty(), splits.get(0));
            for (int i = 0; i < breaks.size(); i++) {
                state = computeBreakAndSplit(commentsHelper, maxWidth, state, Optional.of(breaks.get(i)), splits.get(i + 1));
            }
            return state;
        }
        private static State computeBreakAndSplit(CommentsHelper commentsHelper, int maxWidth, State state, Optional<Break> optBreakDoc, List<Doc> split) {
            int breakWidth = optBreakDoc.isPresent() ? optBreakDoc.get().getWidth() : 0;
            int splitWidth = getWidth(split);
            boolean shouldBreak = optBreakDoc.isPresent() && optBreakDoc.get().fillMode == FillMode.UNIFIED || state.mustBreak || state.column + breakWidth + splitWidth > maxWidth;
            if (optBreakDoc.isPresent()) {
                state = optBreakDoc.get().computeBreaks(state, state.lastIndent, shouldBreak);
            }
            boolean enoughRoom = state.column + splitWidth <= maxWidth;
            state = computeSplit(commentsHelper, maxWidth, split, state.withMustBreak(false));
            if (!enoughRoom) {
                state = state.withMustBreak(true);
            }
            return state;
        }
        private static State computeSplit(CommentsHelper commentsHelper, int maxWidth, List<Doc> docs, State state) {
            for (Doc doc : docs) {
                state = doc.computeBreaks(commentsHelper, maxWidth, state);
            }
            return state;
        }
        @Override
    public void write(Output output) {
            if (oneLine) {
                output.append(getFlat(), range());
            } else {
                writeFilled(output);
            }
        }
        private void writeFilled(Output output) {
            for (Doc doc : splits.get(0)) {
                doc.write(output);
            }
            for (int i = 0; i < breaks.size(); i++) {
                breaks.get(i).write(output);
                for (Doc doc : splits.get(i + 1)) {
                    doc.write(output);
                }
            }
        }
        static int getWidth(List<Doc> docs) {
            int width = 0;
            for (Doc doc : docs) {
                width += doc.getWidth();
                if (width >= MAX_LINE_WIDTH) {
                    return MAX_LINE_WIDTH;
                }
            }
            return width;
        }
        private static Range<Integer> union(Range<Integer> x, Range<Integer> y) {
            return x.isEmpty() ? y : y.isEmpty() ? x : x.span(y).canonical(INTEGERS);
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("plusIndent", plusIndent).add("docs", docs).toString();
        }
    }
    public static final class Token extends Doc implements Op {
        public enum RealOrImaginary {
            REAL, IMAGINARY;
            boolean isReal() {
                return this == REAL;
            }
        }
        private final Input.Token token;
        private final RealOrImaginary realOrImaginary;
        private final Indent plusIndentCommentsBefore;
        private final Optional<Indent> breakAndIndentTrailingComment;
        private Input.Tok tok() {
            return token.getTok();
        }
        private Token(Input.Token token, RealOrImaginary realOrImaginary, Indent plusIndentCommentsBefore, Optional<Indent> breakAndIndentTrailingComment) {
            this.token = token;
            this.realOrImaginary = realOrImaginary;
            this.plusIndentCommentsBefore = plusIndentCommentsBefore;
            this.breakAndIndentTrailingComment = breakAndIndentTrailingComment;
        }
        Indent getPlusIndentCommentsBefore() {
            return plusIndentCommentsBefore;
        }
        Optional<Indent> breakAndIndentTrailingComment() {
            return breakAndIndentTrailingComment;
        }
        static Op make(Input.Token token, Doc.Token.RealOrImaginary realOrImaginary, Indent plusIndentCommentsBefore, Optional<Indent> breakAndIndentTrailingComment) {
            return new Token(token, realOrImaginary, plusIndentCommentsBefore, breakAndIndentTrailingComment);
        }
        Input.Token getToken() {
            return token;
        }
        RealOrImaginary realOrImaginary() {
            return realOrImaginary;
        }
        @Override
    public void add(DocBuilder builder) {
            builder.add(this);
        }
        @Override int computeWidth() {
            int idx = Newlines.firstBreak(tok().getOriginalText());
            return idx >= 0 ? MAX_LINE_WIDTH : tok().length();
        }
        @Override String computeFlat() {
            return token.getTok().getOriginalText();
        }
        @Override Range<Integer> computeRange() {
            return Range.singleton(token.getTok().getIndex()).canonical(INTEGERS);
        }
        @Override
    public State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state) {
            return state.withColumn(state.column + computeWidth());
        }
        @Override
    public void write(Output output) {
            String text = token.getTok().getOriginalText();
            output.append(text, range());
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("token", token).add("realOrImaginary", realOrImaginary).add("plusIndentCommentsBefore", plusIndentCommentsBefore).toString();
        }
    }
    static final class Space extends Doc implements Op {
        private static final Space SPACE = new Space();
        private Space() {}
        static Space make() {
            return SPACE;
        }
        @Override
    public void add(DocBuilder builder) {
            builder.add(this);
        }
        @Override int computeWidth() {
            return 1;
        }
        @Override String computeFlat() {
            return " ";
        }
        @Override Range<Integer> computeRange() {
            return EMPTY_RANGE;
        }
        @Override
    public State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state) {
            return state.withColumn(state.column + 1);
        }
        @Override
    public void write(Output output) {
            output.append(" ", range());
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).toString();
        }
    }
    public static final class Break extends Doc implements Op {
        private final FillMode fillMode;
        private final String flat;
        private final Indent plusIndent;
        private final Optional<BreakTag> optTag;
        private Break(FillMode fillMode, String flat, Indent plusIndent, Optional<BreakTag> optTag) {
            this.fillMode = fillMode;
            this.flat = flat;
            this.plusIndent = plusIndent;
            this.optTag = optTag;
        }
        public static Break make(FillMode fillMode, String flat, Indent plusIndent) {
            return new Break(fillMode, flat, plusIndent, Optional.empty());
        }
        public static Break make(FillMode fillMode, String flat, Indent plusIndent, Optional<BreakTag> optTag) {
            return new Break(fillMode, flat, plusIndent, optTag);
        }
        public static Break makeForced() {
            return make(FillMode.FORCED, "", Indent.Const.ZERO);
        }
        int getPlusIndent() {
            return plusIndent.eval();
        }
        boolean isForced() {
            return fillMode == FillMode.FORCED;
        }
        @Override
    public void add(DocBuilder builder) {
            builder.breakDoc(this);
        }
        @Override int computeWidth() {
            return isForced() ? MAX_LINE_WIDTH : flat.length();
        }
        @Override String computeFlat() {
            return flat;
        }
        @Override Range<Integer> computeRange() {
            return EMPTY_RANGE;
        }
        boolean broken;
        int newIndent;
        public State computeBreaks(State state, int lastIndent, boolean broken) {
            if (optTag.isPresent()) {
                optTag.get().recordBroken(broken);
            }
            if (broken) {
                this.broken = true;
                this.newIndent = max(lastIndent + plusIndent.eval(), 0);
                return state.withColumn(newIndent);
            } else {
                this.broken = false;
                this.newIndent = -1;
                return state.withColumn(state.column + flat.length());
            }
        }
        @Override
    public State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state) {
            throw new UnsupportedOperationException("Did you mean computeBreaks(State, int, boolean)?");
        }
        @Override
    public void write(Output output) {
            if (broken) {
                output.append("\n", EMPTY_RANGE);
                output.indent(newIndent);
            } else {
                output.append(flat, range());
            }
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("fillMode", fillMode).add("flat", flat).add("plusIndent", plusIndent).add("optTag", optTag).toString();
        }
    }
    static final class Tok extends Doc implements Op {
        private final Input.Tok tok;
        private Tok(Input.Tok tok) {
            this.tok = tok;
        }
        static Tok make(Input.Tok tok) {
            return new Tok(tok);
        }
        @Override
    public void add(DocBuilder builder) {
            builder.add(this);
        }
        @Override int computeWidth() {
            int idx = Newlines.firstBreak(tok.getOriginalText());
            return tok.isComment() ? idx > 0 ? idx : tok.isSlashSlashComment() && !tok.getOriginalText().startsWith("// ") ? tok.length() + 1 : reformatParameterComment(tok).map(String::length).orElse(tok.length()) : idx != -1 ? MAX_LINE_WIDTH : tok.length();
        }
        @Override String computeFlat() {
            return tok.isSlashSlashComment() && !tok.getOriginalText().startsWith("// ") ? "// " + tok.getOriginalText().substring(2) : reformatParameterComment(tok).orElse(tok.getOriginalText());
        }
        @Override Range<Integer> computeRange() {
            return Range.singleton(tok.getIndex()).canonical(INTEGERS);
        }
        String text;
        @Override
    public State computeBreaks(CommentsHelper commentsHelper, int maxWidth, State state) {
            text = commentsHelper.rewrite(tok, maxWidth, state.column);
            int firstLineLength = text.length() - Iterators.getLast(Newlines.lineOffsetIterator(text));
            return state.withColumn(state.column + firstLineLength);
        }
        @Override
    public void write(Output output) {
            output.append(text, range());
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("tok", tok).toString();
        }
    }
}
