package com.google.googlejavaformat.java;

import static com.google.common.collect.ImmutableList.toImmutableList;
import static com.google.common.collect.Iterables.getLast;
import static com.google.googlejavaformat.java.Trees.getEndPosition;
import static com.google.googlejavaformat.java.Trees.getStartPosition;
import static java.lang.Math.min;
import static java.util.stream.Collectors.joining;
import com.google.common.base.CharMatcher;
import com.google.common.base.Verify;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import com.google.common.collect.TreeRangeMap;
import com.google.googlejavaformat.Newlines;
import com.sun.source.tree.BinaryTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.Tree.Kind;
import com.sun.source.util.TreePath;
import com.sun.source.util.TreePathScanner;
import com.sun.tools.javac.tree.JCTree;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.Position;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicBoolean;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;

public final class StringWrapper {
    private static final String TEXT_BLOCK_DELIMITER = "\"\"\"";
    public static String wrap(String input, Formatter formatter) throws FormatterException {
        return StringWrapper.wrap(formatter.options().maxLineLength(), input, formatter);
    }
    static String wrap(final int columnLimit, String input, Formatter formatter) throws FormatterException {
        JavaFormatterOptions options = formatter.options();
        if (!needWrapping(columnLimit, input, options)) {
            return input;
        }
        TreeRangeMap<Integer, String> replacements = getReflowReplacements(columnLimit, input, options);
        String firstPass = formatter.formatSource(input, replacements.asMapOfRanges().keySet());
        if (!firstPass.equals(input)) {
            input = firstPass;
            replacements = getReflowReplacements(columnLimit, input, options);
        }
        String result = applyReplacements(input, replacements);
        {
            String expected = parse(input, true).toString();
            String actual = parse(result, true).toString();
            if (!expected.equals(actual)) {
                throw new FormatterException(String.format("Something has gone terribly wrong. We planned to make the below formatting change, but have aborted because it would unexpectedly change the AST.\nPlease file a bug: https://github.com/google/google-java-format/issues/new\n\n=== Actual: ===\n%s\n=== Expected: ===\n%s\n", actual, expected));
            }
        }
        return result;
    }
    private static TreeRangeMap<Integer, String> getReflowReplacements(int columnLimit, final String input, JavaFormatterOptions options) throws FormatterException {
        return new Reflower(columnLimit, input, options).getReflowReplacements();
    }
    private static class Reflower {
        private final String input;
        private final int columnLimit;
        private final JavaFormatterOptions options;
        private final String separator;
        private final JCTree.JCCompilationUnit unit;
        private final Position.LineMap lineMap;
        Reflower(int columnLimit, String input, JavaFormatterOptions options) throws FormatterException {
            this.columnLimit = columnLimit;
            this.input = input;
            this.options = options;
            this.separator = Newlines.guessLineSeparator(input);
            this.unit = parse(input, false);
            this.lineMap = unit.getLineMap();
        }
        private int visualColumn(int position) {
            int lineStart = lineMap.getStartPosition(lineMap.getLineNumber(position));
            return options.visualLength(input, lineStart, position);
        }
        TreeRangeMap<Integer, String> getReflowReplacements() {
            List<TreePath> longStringLiterals = new ArrayList<>();
            List<Tree> textBlocks = new ArrayList<>();
            new LongStringsAndTextBlockScanner(longStringLiterals, textBlocks).scan(new TreePath(unit), null);
            TreeRangeMap<Integer, String> replacements = TreeRangeMap.create();
            indentTextBlocks(replacements, textBlocks);
            wrapLongStrings(replacements, longStringLiterals);
            return replacements;
        }
        private class LongStringsAndTextBlockScanner extends TreePathScanner<Void, Void> {
            private final List<TreePath> longStringLiterals;
            private final List<Tree> textBlocks;
            LongStringsAndTextBlockScanner(List<TreePath> longStringLiterals, List<Tree> textBlocks) {
                this.longStringLiterals = longStringLiterals;
                this.textBlocks = textBlocks;
            }
            @Override
      public Void visitLiteral(LiteralTree literalTree, Void aVoid) {
                if (literalTree.getKind() != Kind.STRING_LITERAL) {
                    return null;
                }
                int pos = getStartPosition(literalTree);
                if (input.substring(pos, min(input.length(), pos + 3)).equals(TEXT_BLOCK_DELIMITER)) {
                    textBlocks.add(literalTree);
                    return null;
                }
                Tree parent = getCurrentPath().getParentPath().getLeaf();
                if (parent instanceof MemberSelectTree memberSelectTree && memberSelectTree.getExpression().equals(literalTree)) {
                    return null;
                }
                int lineEnd = getEndPosition(literalTree, unit);
                while (Newlines.hasNewlineAt(input, lineEnd) == -1) {
                    lineEnd++;
                }
                if (visualColumn(lineEnd) <= columnLimit) {
                    return null;
                }
                longStringLiterals.add(getCurrentPath());
                return null;
            }
        }
        private void indentTextBlocks(TreeRangeMap<Integer, String> replacements, List<Tree> textBlocks) {
            for (Tree tree : textBlocks) {
                int startPosition = lineMap.getStartPosition(lineMap.getLineNumber(getStartPosition(tree)));
                int endPosition = getEndPosition(tree, unit);
                String text = input.substring(startPosition, endPosition);
                int leadingWhitespace = CharMatcher.whitespace().negate().indexIn(text);
                ImmutableList<String> initialLines = text.lines().collect(toImmutableList());
                ImmutableList<String> lines = initialLines.stream().skip(1).collect(joining(separator)).stripIndent().lines().collect(toImmutableList());
                boolean deindent = getLast(initialLines).stripTrailing().length() == getLast(lines).stripTrailing().length();
                String prefix = deindent ? "" : options.indentString(visualColumn(startPosition + leadingWhitespace));
                StringBuilder output = new StringBuilder(prefix).append(initialLines.get(0).stripLeading());
                for (int i = 0; i < lines.size(); i++) {
                    String line = lines.get(i);
                    String trimmed = line.stripTrailing();
                    output.append(separator);
                    if (!trimmed.isEmpty()) {
                        output.append(prefix);
                    }
                    if (i == lines.size() - 1) {
                        String withoutDelimiter = trimmed.substring(0, trimmed.length() - TEXT_BLOCK_DELIMITER.length()).stripTrailing();
                        if (!withoutDelimiter.stripLeading().isEmpty()) {
                            output.append(withoutDelimiter).append('\\').append(separator).append(prefix);
                        }
                        output.append(TEXT_BLOCK_DELIMITER);
                    } else {
                        output.append(line);
                    }
                }
                replacements.put(Range.closedOpen(startPosition, endPosition), output.toString());
            }
        }
        private void wrapLongStrings(TreeRangeMap<Integer, String> replacements, List<TreePath> longStringLiterals) {
            for (TreePath path : longStringLiterals) {
                TreePath enclosing = path;
                while (enclosing.getParentPath().getLeaf().getKind() == Kind.PLUS) {
                    enclosing = enclosing.getParentPath();
                }
                AtomicBoolean first = new AtomicBoolean(false);
                List<Tree> flat = flatten(input, unit, path, enclosing, first);
                int startColumn = visualColumn(getStartPosition(flat.get(0)));
                int end = getEndPosition(getLast(flat), unit);
                int lineEnd = end;
                while (Newlines.hasNewlineAt(input, lineEnd) == -1) {
                    lineEnd++;
                }
                ImmutableList<String> components = stringComponents(input, unit, flat);
                replacements.put(Range.closedOpen(getStartPosition(flat.get(0)), getEndPosition(getLast(flat), unit)), reflow(separator, columnLimit, startColumn, lineEnd - end, components, first.get(), options));
            }
        }
    }
    private static ImmutableList<String> stringComponents(String input, JCTree.JCCompilationUnit unit, List<Tree> flat) {
        ImmutableList.Builder<String> result = ImmutableList.builder();
        StringBuilder piece = new StringBuilder();
        for (Tree tree : flat) {
            String text = input.substring(getStartPosition(tree) + 1, getEndPosition(tree, unit) - 1);
            int start = 0;
            for (int idx = 0; idx < text.length(); idx++) {
                if (!CharMatcher.whitespace().matches(text.charAt(idx))) 
                    if (hasEscapedWhitespaceAt(text, idx) == -1) 
                        if (hasEscapedNewlineAt(text, idx) != -1) {
                            int length;
                            while ((length = hasEscapedNewlineAt(text, idx)) != -1) {
                                idx += length;
                            }
                        } else {
                            continue;
                        }
                piece.append(text, start, idx);
                result.add(piece.toString());
                piece = new StringBuilder();
                start = idx;
            }
            if (piece.length() > 0) {
                result.add(piece.toString());
                piece = new StringBuilder();
            }
            if (start < text.length()) {
                piece.append(text, start, text.length());
            }
        }
        if (piece.length() > 0) {
            result.add(piece.toString());
        }
        return result.build();
    }
    private static int hasEscapedWhitespaceAt(String input, int idx) {
        return input.startsWith("\\t", idx) ? 2 : -1;
    }
    private static int hasEscapedNewlineAt(String input, int idx) {
        int offset = 0;
        if (input.startsWith("\\r", idx)) {
            offset += 2;
        }
        if (input.startsWith("\\n", idx)) {
            offset += 2;
        }
        return offset > 0 ? offset : -1;
    }
    private static String reflow(String separator, int columnLimit, int startColumn, int trailing, ImmutableList<String> components, boolean first0, JavaFormatterOptions options) {
        int width = columnLimit - startColumn - 2;
        Deque<String> input = new ArrayDeque<>(components);
        List<String> lines = new ArrayList<>();
        boolean first = first0;
        while (!input.isEmpty()) {
            int length = 0;
            List<String> line = new ArrayList<>();
            if (totalLengthLessThanOrEqual(input, width)) {
                width -= trailing;
            }
            while (!input.isEmpty() && (length <= 4 || length + input.peekFirst().length() <= width)) {
                String text = input.removeFirst();
                line.add(text);
                length += text.length();
                if (text.endsWith("\\n") || text.endsWith("\\r")) {
                    break;
                }
            }
            if (line.isEmpty()) {
                line.add(input.removeFirst());
            }
            lines.add(String.join("", line));
            if (first) {
                width -= 6;
                first = false;
            }
        }
        return lines.stream().collect(joining("\"" + separator + options.indentString(startColumn + (first0 ? 4 : -2)) + "+ \"", "\"", "\""));
    }
    private static boolean totalLengthLessThanOrEqual(Iterable<String> input, int length) {
        int total = 0;
        for (String s : input) {
            total += s.length();
            if (total > length) {
                return false;
            }
        }
        return true;
    }
    private static List<Tree> flatten(String input, JCTree.JCCompilationUnit unit, TreePath path, TreePath parent, AtomicBoolean firstInChain) {
        List<Tree> flat = new ArrayList<>();
        ArrayDeque<Tree> todo = new ArrayDeque<>();
        todo.add(parent.getLeaf());
        while (!todo.isEmpty()) {
            Tree first = todo.removeFirst();
            if (first.getKind() == Tree.Kind.PLUS) {
                BinaryTree bt = (BinaryTree) first;
                todo.addFirst(bt.getRightOperand());
                todo.addFirst(bt.getLeftOperand());
            } else {
                flat.add(first);
            }
        }
        int idx = flat.indexOf(path.getLeaf());
        Verify.verify(idx != -1);
        int startIdx = idx;
        int endIdx = idx + 1;
        while (startIdx > 0 && flat.get(startIdx - 1).getKind() == Tree.Kind.STRING_LITERAL && noComments(input, unit, flat.get(startIdx - 1), flat.get(startIdx))) {
            startIdx--;
        }
        while (endIdx < flat.size() && flat.get(endIdx).getKind() == Tree.Kind.STRING_LITERAL && noComments(input, unit, flat.get(endIdx - 1), flat.get(endIdx))) {
            endIdx++;
        }
        firstInChain.set(startIdx == 0);
        return ImmutableList.copyOf(flat.subList(startIdx, endIdx));
    }
    private static boolean noComments(String input, JCTree.JCCompilationUnit unit, Tree one, Tree two) {
        return STRING_CONCAT_DELIMITER.matchesAllOf(input.subSequence(getEndPosition(one, unit), getStartPosition(two)));
    }
    private static final CharMatcher STRING_CONCAT_DELIMITER = CharMatcher.whitespace().or(CharMatcher.anyOf("\"+"));
    private static boolean needWrapping(int columnLimit, String input, JavaFormatterOptions options) {
        Iterator<String> it = Newlines.lineIterator(input);
        while (it.hasNext()) {
            String line = it.next();
            if (options.visualLength(line) > columnLimit || line.contains(TEXT_BLOCK_DELIMITER)) {
                return true;
            }
        }
        return false;
    }
    private static JCTree.JCCompilationUnit parse(String source, boolean allowStringFolding) throws FormatterException {
        List<Diagnostic<? extends JavaFileObject>> errorDiagnostics = new ArrayList<>();
        Context context = new Context();
        JCTree.JCCompilationUnit unit = Trees.parse(context, errorDiagnostics, allowStringFolding, source);
        if (!errorDiagnostics.isEmpty()) {
            throw FormatterException.fromJavacDiagnostics(errorDiagnostics);
        }
        return unit;
    }
    private static String applyReplacements(String javaInput, TreeRangeMap<Integer, String> replacementMap) throws FormatterException {
        Map<Range<Integer>, String> ranges = replacementMap.asDescendingMapOfRanges();
        if (ranges.isEmpty()) {
            return javaInput;
        }
        StringBuilder sb = new StringBuilder(javaInput);
        for (Map.Entry<Range<Integer>, String> entry : ranges.entrySet()) {
            Range<Integer> range = entry.getKey();
            sb.replace(range.lowerEndpoint(), range.upperEndpoint(), entry.getValue());
        }
        return sb.toString();
    }
    private StringWrapper() {}
}
