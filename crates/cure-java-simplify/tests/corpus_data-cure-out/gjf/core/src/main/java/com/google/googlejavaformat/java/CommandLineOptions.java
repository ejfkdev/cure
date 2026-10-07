package com.google.googlejavaformat.java;

import com.google.auto.value.AutoBuilder;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableRangeSet;
import com.google.errorprone.annotations.CanIgnoreReturnValue;
import java.util.Optional;

record CommandLineOptions(
    ImmutableList<String> files,
    boolean inPlace,
    ImmutableRangeSet<Integer> lines,
    ImmutableList<Integer> offsets,
    ImmutableList<Integer> lengths,
    JavaFormatterOptions.Style style,
    boolean version,
    boolean help,
    boolean stdin,
    boolean fixImportsOnly,
    boolean sortImports,
    boolean removeUnusedImports,
    boolean dryRun,
    boolean setExitIfChanged,
    Optional<String> assumeFilename,
    boolean reflowLongStrings,
    boolean formatJavadoc,
    boolean reorderModifiers) {
    boolean aosp() {
        return style().isAosp();
    }
    int maxLineLength() {
        return style().maxLineLength();
    }
    boolean isSelection() {
        return !lines().isEmpty() || !offsets().isEmpty() || !lengths().isEmpty();
    }
    static Builder builder() {
        return new AutoBuilder_CommandLineOptions_Builder().style(JavaFormatterOptions.Style.GOOGLE).sortImports(true).removeUnusedImports(true).reflowLongStrings(true).formatJavadoc(true).reorderModifiers(true).version(false).help(false).stdin(false).fixImportsOnly(false).dryRun(false).setExitIfChanged(false).inPlace(false);
    }
    @AutoBuilder interface Builder {
        ImmutableList.Builder<String> filesBuilder();
        Builder inPlace(boolean inPlace);
        Builder lines(ImmutableRangeSet<Integer> lines);
        ImmutableList.Builder<Integer> offsetsBuilder();
        @CanIgnoreReturnValue
    default Builder addOffset(Integer offset) {
            offsetsBuilder().add(offset);
            return this;
        }
        ImmutableList.Builder<Integer> lengthsBuilder();
        @CanIgnoreReturnValue
    default Builder addLength(Integer length) {
            lengthsBuilder().add(length);
            return this;
        }
        Builder style(JavaFormatterOptions.Style style);
        Builder version(boolean version);
        Builder help(boolean help);
        Builder stdin(boolean stdin);
        Builder fixImportsOnly(boolean fixImportsOnly);
        Builder sortImports(boolean sortImports);
        Builder removeUnusedImports(boolean removeUnusedImports);
        Builder dryRun(boolean dryRun);
        Builder setExitIfChanged(boolean setExitIfChanged);
        Builder assumeFilename(String assumeFilename);
        Builder reflowLongStrings(boolean reflowLongStrings);
        Builder formatJavadoc(boolean formatJavadoc);
        Builder reorderModifiers(boolean reorderModifiers);
        CommandLineOptions build();
    }
}
