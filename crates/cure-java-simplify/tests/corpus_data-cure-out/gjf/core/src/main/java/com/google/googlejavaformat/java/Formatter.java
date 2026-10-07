package com.google.googlejavaformat.java;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Iterators;
import com.google.common.collect.Range;
import com.google.common.collect.RangeSet;
import com.google.common.collect.TreeRangeSet;
import com.google.common.io.CharSink;
import com.google.common.io.CharSource;
import com.google.errorprone.annotations.Immutable;
import com.google.googlejavaformat.CommentsHelper;
import com.google.googlejavaformat.Doc;
import com.google.googlejavaformat.DocBuilder;
import com.google.googlejavaformat.FormattingError;
import com.google.googlejavaformat.Newlines;
import com.google.googlejavaformat.Op;
import com.google.googlejavaformat.OpsBuilder;
import com.sun.tools.javac.tree.JCTree.JCCompilationUnit;
import com.sun.tools.javac.util.Context;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;

@Immutable
public final class Formatter {
    public static final int MAX_LINE_LENGTH = 100;
    static final Range<Integer> EMPTY_RANGE = Range.closedOpen(-1, -1);
    private final JavaFormatterOptions options;
    public Formatter() {
        this(JavaFormatterOptions.defaultOptions());
    }
    public Formatter(JavaFormatterOptions options) {
        this.options = options;
    }
    JavaFormatterOptions options() {
        return options;
    }
    static void format(final JavaInput javaInput, JavaOutput javaOutput, JavaFormatterOptions options) throws FormatterException {
        Context context = new Context();
        List<Diagnostic<? extends JavaFileObject>> errorDiagnostics = new ArrayList<>();
        JCCompilationUnit unit = Trees.parse(context, errorDiagnostics, false, javaInput.getText());
        javaInput.setCompilationUnit(unit);
        if (!errorDiagnostics.isEmpty()) {
            throw FormatterException.fromJavacDiagnostics(errorDiagnostics);
        }
        OpsBuilder builder = new OpsBuilder(javaInput, javaOutput);
        ImmutableSet.Builder<Integer> markdownJavadocPositions = ImmutableSet.builder();
        new JavaInputAstVisitor(builder, options.indentationMultiplier(), markdownJavadocPositions).scan(unit, null);
        builder.sync(javaInput.getText().length());
        builder.drain();
        Doc doc = new DocBuilder().withOps(builder.build()).build();
        CommentsHelper commentsHelper = new JavaCommentsHelper(Newlines.guessLineSeparator(javaInput.getText()), options, markdownJavadocPositions.build());
        doc.computeBreaks(commentsHelper, options.maxLineLength(), new Doc.State(0, 0));
        doc.write(javaOutput);
        javaOutput.flush();
    }
    static boolean errorDiagnostic(Diagnostic<?> input) {
        return input.getKind() != Diagnostic.Kind.ERROR ? false : !input.getCode().equals("compiler.err.invalid.meth.decl.ret.type.req");
    }
    public void formatSource(CharSource input, CharSink output) throws FormatterException, IOException {
        output.write(formatSource(input.read()));
    }
    public String formatSource(String input) throws FormatterException {
        return formatSource(input, ImmutableList.of(Range.closedOpen(0, input.length())));
    }
    public String formatSourceAndFixImports(String input) throws FormatterException {
        input = ImportOrderer.reorderImports(input, options.style());
        input = RemoveUnusedImports.removeUnusedImports(input);
        String formatted = formatSource(input);
        formatted = StringWrapper.wrap(formatted, this);
        return formatted;
    }
    public String formatSource(String input, Collection<Range<Integer>> characterRanges) throws FormatterException {
        return JavaOutput.applyReplacements(input, getFormatReplacements(input, characterRanges));
    }
    public ImmutableList<Replacement> getFormatReplacements(String input, Collection<Range<Integer>> characterRanges) throws FormatterException {
        JavaInput javaInput = new JavaInput(input);
        if (options.reorderModifiers()) {
            javaInput = ModifierOrderer.reorderModifiers(javaInput, characterRanges);
        }
        String lineSeparator = Newlines.guessLineSeparator(input);
        JavaOutput javaOutput = new JavaOutput(lineSeparator, javaInput, new JavaCommentsHelper(lineSeparator, options, ImmutableSet.of()), options::indentString);
        try {
            format(javaInput, javaOutput, options);
        } catch (FormattingError e) {
            throw new FormatterException(e.diagnostics());
        }
        RangeSet<Integer> tokenRangeSet = javaInput.characterRangesToTokenRanges(characterRanges);
        return javaOutput.getFormatReplacements(tokenRangeSet);
    }
    public static RangeSet<Integer> lineRangesToCharRanges(String input, RangeSet<Integer> lineRanges) {
        List<Integer> lines = new ArrayList<>();
        Iterators.addAll(lines, Newlines.lineOffsetIterator(input));
        lines.add(input.length() + 1);
        RangeSet<Integer> characterRanges = TreeRangeSet.create();
        for (Range<Integer> lineRange : lineRanges.subRangeSet(Range.closedOpen(0, lines.size() - 1)).asRanges()) {
            int lineStart = lines.get(lineRange.lowerEndpoint());
            int lineEnd = lines.get(lineRange.upperEndpoint()) - 1;
            Range<Integer> range = Range.closedOpen(lineStart, lineEnd);
            characterRanges.add(range);
        }
        return characterRanges;
    }
}
