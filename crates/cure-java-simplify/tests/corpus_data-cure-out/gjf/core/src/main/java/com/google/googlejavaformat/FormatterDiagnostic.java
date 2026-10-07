package com.google.googlejavaformat;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

public record FormatterDiagnostic(int line, int column, String message) {
    public FormatterDiagnostic {
        checkArgument(line >= -1);
        checkArgument(column >= -1);
        checkNotNull(message);
    }
    public FormatterDiagnostic(String message) {
        this(-1, -1, message);
    }
    @Override
  public String toString() {
        StringBuilder sb = new StringBuilder();
        if (line >= 0) {
            sb.append(line).append(':');
        }
        if (column >= 0) {
            sb.append(column).append(':');
        }
        if (line >= 0 || column >= 0) {
            sb.append(' ');
        }
        sb.append("error: ").append(message);
        return sb.toString();
    }
}
