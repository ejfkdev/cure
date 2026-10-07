package com.google.googlejavaformat;

import com.google.common.base.Joiner;
import com.google.common.collect.ImmutableList;

public class FormattingError extends Error {
    private final ImmutableList<FormatterDiagnostic> diagnostics;
    public FormattingError(FormatterDiagnostic diagnostic) {
        this(ImmutableList.of(diagnostic));
    }
    public FormattingError(Iterable<FormatterDiagnostic> diagnostics) {
        super(Joiner.on("\n").join(diagnostics) + "\n");
        this.diagnostics = ImmutableList.copyOf(diagnostics);
    }
    public ImmutableList<FormatterDiagnostic> diagnostics() {
        return diagnostics;
    }
}
