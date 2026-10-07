package com.google.googlejavaformat.java;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import com.google.common.collect.Range;
import com.google.errorprone.annotations.InlineMe;

public record Replacement(Range<Integer> replaceRange, String replacementString) {
    public static Replacement create(int startPosition, int endPosition, String replaceWith) {
        checkArgument(startPosition >= 0, "startPosition must be non-negative");
        checkArgument(startPosition <= endPosition, "startPosition cannot be after endPosition");
        return new Replacement(Range.closedOpen(startPosition, endPosition), replaceWith);
    }
    public Replacement {
        checkNotNull(replaceRange, "Null replaceRange");
        checkNotNull(replacementString, "Null replacementString");
    }
    @InlineMe(replacement = "this.replaceRange()")
  public Range<Integer> getReplaceRange() {
        return replaceRange();
    }
    @InlineMe(replacement = "this.replacementString()")
  public String getReplacementString() {
        return replacementString();
    }
}
