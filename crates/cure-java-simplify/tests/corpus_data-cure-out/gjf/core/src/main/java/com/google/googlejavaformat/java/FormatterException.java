package com.google.googlejavaformat.java;

import static com.google.common.collect.ImmutableList.toImmutableList;
import static java.util.Locale.ENGLISH;
import com.google.common.base.CharMatcher;
import com.google.common.base.Splitter;
import com.google.common.collect.ImmutableList;
import com.google.googlejavaformat.FormatterDiagnostic;
import java.util.List;
import java.util.regex.Pattern;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;

public final class FormatterException extends Exception {
    private final ImmutableList<FormatterDiagnostic> diagnostics;
    public FormatterException(String message) {
        this(new FormatterDiagnostic(message));
    }
    public FormatterException(FormatterDiagnostic diagnostic) {
        this(ImmutableList.of(diagnostic));
    }
    public FormatterException(Iterable<FormatterDiagnostic> diagnostics) {
        super(diagnostics.iterator().next().toString());
        this.diagnostics = ImmutableList.copyOf(diagnostics);
    }
    public List<FormatterDiagnostic> diagnostics() {
        return diagnostics;
    }
    public static FormatterException fromJavacDiagnostics(List<Diagnostic<? extends JavaFileObject>> diagnostics) {
        return new FormatterException(diagnostics.stream().map(FormatterException::toFormatterDiagnostic).collect(toImmutableList()));
    }
    private static FormatterDiagnostic toFormatterDiagnostic(Diagnostic<?> input) {
        return new FormatterDiagnostic((int) input.getLineNumber(), (int) input.getColumnNumber(), input.getMessage(ENGLISH));
    }
    public String formatDiagnostics(String path, String input) {
        List<String> lines = Splitter.on(NEWLINE_PATTERN).splitToList(input);
        StringBuilder sb = new StringBuilder();
        for (FormatterDiagnostic diagnostic : diagnostics()) {
            sb.append(path).append(":").append(diagnostic).append(System.lineSeparator());
            int line = diagnostic.line();
            int column = diagnostic.column();
            if (line != -1 && column != -1) {
                sb.append(CharMatcher.breakingWhitespace().trimTrailingFrom(lines.get(line - 1))).append(System.lineSeparator());
                sb.append(" ".repeat(column - 1)).append('^').append(System.lineSeparator());
            }
        }
        return sb.toString();
    }
    private static final Pattern NEWLINE_PATTERN = Pattern.compile("\\R");
}
