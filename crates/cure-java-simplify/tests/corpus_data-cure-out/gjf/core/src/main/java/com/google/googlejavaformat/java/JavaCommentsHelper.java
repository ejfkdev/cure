package com.google.googlejavaformat.java;

import com.google.common.base.CharMatcher;
import com.google.common.collect.ImmutableSet;
import com.google.googlejavaformat.CommentsHelper;
import com.google.googlejavaformat.Input.Tok;
import com.google.googlejavaformat.Newlines;
import com.google.googlejavaformat.java.javadoc.JavadocFormatter;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

final class JavaCommentsHelper implements CommentsHelper {
    private final String lineSeparator;
    private final JavaFormatterOptions options;
    private final ImmutableSet<Integer> markdownJavadocPositions;
    JavaCommentsHelper(String lineSeparator, JavaFormatterOptions options, ImmutableSet<Integer> markdownJavadocPositions) {
        this.lineSeparator = lineSeparator;
        this.options = options;
        this.markdownJavadocPositions = markdownJavadocPositions;
    }
    @Override
  public String rewrite(Tok tok, int maxWidth, int column0) {
        if (!tok.isComment()) {
            return tok.getOriginalText();
        }
        String text = tok.getOriginalText();
        if (tok.isJavadocComment() && options.formatJavadoc()) {
            if (text.startsWith("///")) {
                if (markdownJavadocPositions.contains(tok.getPosition())) {
                    return JavadocFormatter.formatJavadoc(text, column0, options.maxLineLength());
                }
            } else {
                text = JavadocFormatter.formatJavadoc(text, column0, options.maxLineLength());
            }
        }
        List<String> lines = new ArrayList<>();
        Iterator<String> it = Newlines.lineIterator(text);
        while (it.hasNext()) {
            if (tok.isSlashSlashComment()) {
                lines.add(CharMatcher.whitespace().trimFrom(it.next()));
            } else {
                lines.add(CharMatcher.whitespace().trimTrailingFrom(it.next()));
            }
        }
        return tok.isSlashSlashComment() ? indentLineComments(tok, lines, column0) : CommentsHelper.reformatParameterComment(tok).orElseGet(() -> javadocShaped(lines) ? indentJavadoc(lines, column0) : preserveIndentation(lines, column0));
    }
    private String preserveIndentation(List<String> lines, int column0) {
        StringBuilder builder = new StringBuilder();
        int startCol = -1;
        for (int i = 1; i < lines.size(); i++) {
            int lineIdx = CharMatcher.whitespace().negate().indexIn(lines.get(i));
            if (lineIdx >= 0 && (startCol == -1 || lineIdx < startCol)) {
                startCol = lineIdx;
            }
        }
        builder.append(lines.get(0));
        String indentString = options.indentString(column0);
        for (int i = 1; i < lines.size(); ++i) {
            builder.append(lineSeparator).append(indentString);
            if (lines.get(i).length() >= startCol) {
                builder.append(lines.get(i).substring(startCol));
            } else {
                builder.append(lines.get(i));
            }
        }
        return builder.toString();
    }
    private String indentLineComments(Tok tok, List<String> lines, int column0) {
        lines = wrapLineComments(tok, lines, column0);
        StringBuilder builder = new StringBuilder();
        builder.append(lines.get(0).trim());
        String indentString = options.indentString(column0);
        for (int i = 1; i < lines.size(); ++i) {
            builder.append(lineSeparator).append(indentString).append(lines.get(i).trim());
        }
        return builder.toString();
    }
    private static final Pattern LINE_COMMENT_MISSING_SPACE_PREFIX = Pattern.compile("^(//+)[^\\s/]");
    private static final Pattern LINE_COMMENT_NO_SPACE_PREFIX = Pattern.compile("^//+(noinspection|\\$NON-NLS-\\d+\\$)");
    private List<String> wrapLineComments(Tok tok, List<String> lines, int column0) {
        if (markdownJavadocPositions.contains(tok.getPosition())) {
            return lines;
        }
        List<String> result = new ArrayList<>();
        for (String line : lines) {
            Matcher matcher = LINE_COMMENT_MISSING_SPACE_PREFIX.matcher(line);
            if (matcher.find() && !LINE_COMMENT_NO_SPACE_PREFIX.matcher(line).find()) {
                int length = matcher.group(1).length();
                line = "/".repeat(length) + " " + line.substring(length);
            }
            if (line.startsWith("// MOE:")) {
                result.add(line);
                continue;
            }
            while (line.length() + column0 > options.maxLineLength()) {
                int idx = options.maxLineLength() - column0;
                while (idx >= 2 && !CharMatcher.whitespace().matches(line.charAt(idx))) {
                    idx--;
                }
                if (idx <= 2) {
                    break;
                }
                result.add(line.substring(0, idx));
                line = "//" + line.substring(idx);
            }
            result.add(line);
        }
        return result;
    }
    private String indentJavadoc(List<String> lines, int column0) {
        StringBuilder builder = new StringBuilder();
        builder.append(lines.get(0).trim());
        String indentString = options.indentString(column0 + 1);
        for (int i = 1; i < lines.size(); ++i) {
            builder.append(lineSeparator).append(indentString);
            String line = lines.get(i).trim();
            if (!line.startsWith("*")) {
                builder.append("* ");
            }
            builder.append(line);
        }
        return builder.toString();
    }
    private static boolean javadocShaped(List<String> lines) {
        Iterator<String> it = lines.iterator();
        if (!it.hasNext()) {
            return false;
        }
        String first = it.next().trim();
        if (first.startsWith("/**")) {
            return true;
        }
        if (!first.startsWith("/*")) {
            return false;
        }
        while (it.hasNext()) {
            if (!it.next().trim().startsWith("*")) {
                return false;
            }
        }
        return true;
    }
}
