package com.google.googlejavaformat.java;

import static com.google.common.base.Preconditions.checkArgument;
import static java.util.Objects.requireNonNull;
import com.google.auto.value.AutoBuilder;
import com.google.errorprone.annotations.CanIgnoreReturnValue;
import com.google.errorprone.annotations.Immutable;
import com.google.googlejavaformat.Doc;

@Immutable
public record JavaFormatterOptions(boolean formatJavadoc, boolean reorderModifiers, Style style) {
    public JavaFormatterOptions {
        requireNonNull(style, "style");
    }
    @Immutable
  public record Style(
      int indentationMultiplier, int maxLineLength, boolean useTabs, ImportOrder importOrder) {
        public Style {
            checkArgument(maxLineLength > 0 && maxLineLength <= MAX_LINE_LENGTH_LIMIT, "maxLineLength must be between 1 and %s, was: %s", MAX_LINE_LENGTH_LIMIT, maxLineLength);
            requireNonNull(importOrder, "importOrder");
        }
        static final int MAX_LINE_LENGTH_LIMIT = Doc.MAX_LINE_WIDTH - 1;
        public static final Style GOOGLE = builder().google().build();
        public static final Style AOSP = builder().aosp().build();
        public int tabWidth() {
            return 2 * indentationMultiplier();
        }
        public String indentString(int indent) {
            if (!useTabs()) {
                return JavaOutput.spaces(indent);
            }
            int tabWidth = tabWidth();
            return "\t".repeat(indent / tabWidth) + " ".repeat(indent % tabWidth);
        }
        public int visualLength(CharSequence input) {
            return visualLength(input, 0, input.length());
        }
        public int visualLength(CharSequence input, int start, int end) {
            if (!useTabs()) {
                return end - start;
            }
            int tabWidth = tabWidth();
            int column = 0;
            for (int i = start; i < end; i++) {
                if (input.charAt(i) == '\t') {
                    column += tabWidth - column % tabWidth;
                } else {
                    column++;
                }
            }
            return column;
        }
        public boolean isAosp() {
            return importOrder() == ImportOrder.AOSP;
        }
        public static Builder builder() {
            return new AutoBuilder_JavaFormatterOptions_Style_Builder().maxLineLength(100).useTabs(false).google();
        }
        public Builder toBuilder() {
            return new AutoBuilder_JavaFormatterOptions_Style_Builder().indentationMultiplier(indentationMultiplier()).maxLineLength(maxLineLength()).useTabs(useTabs()).importOrder(importOrder());
        }
        @AutoBuilder
    public abstract static class Builder {
            public abstract Builder indentationMultiplier(int indentationMultiplier);
            public abstract Builder maxLineLength(int maxLineLength);
            public abstract Builder useTabs(boolean useTabs);
            public abstract Builder importOrder(ImportOrder importOrder);
            @CanIgnoreReturnValue
      public Builder aosp() {
                return indentationMultiplier(2).importOrder(ImportOrder.AOSP);
            }
            @CanIgnoreReturnValue
      public Builder google() {
                return indentationMultiplier(1).importOrder(ImportOrder.GOOGLE);
            }
            public abstract Style build();
        }
    }
    public enum ImportOrder {
        GOOGLE, AOSP
    }
    public int indentationMultiplier() {
        return style().indentationMultiplier();
    }
    public int maxLineLength() {
        return style().maxLineLength();
    }
    public boolean useTabs() {
        return style().useTabs();
    }
    public String indentString(int indent) {
        return style().indentString(indent);
    }
    public int visualLength(CharSequence input) {
        return style().visualLength(input);
    }
    public int visualLength(CharSequence input, int start, int end) {
        return style().visualLength(input, start, end);
    }
    public static JavaFormatterOptions defaultOptions() {
        return builder().build();
    }
    public static Builder builder() {
        return new AutoBuilder_JavaFormatterOptions_Builder().style(Style.GOOGLE).formatJavadoc(true).reorderModifiers(true);
    }
    @AutoBuilder
  public abstract static class Builder {
        public abstract Builder style(Style style);
        public abstract Builder formatJavadoc(boolean formatJavadoc);
        public abstract Builder reorderModifiers(boolean reorderModifiers);
        public abstract JavaFormatterOptions build();
    }
}
