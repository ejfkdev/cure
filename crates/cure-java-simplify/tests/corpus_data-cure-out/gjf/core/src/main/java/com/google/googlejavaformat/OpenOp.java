package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;

public final class OpenOp implements Op {
    private final Indent plusIndent;
    private OpenOp(Indent plusIndent) {
        this.plusIndent = plusIndent;
    }
    public static Op make(Indent plusIndent) {
        return new OpenOp(plusIndent);
    }
    @Override
  public void add(DocBuilder builder) {
        builder.open(plusIndent);
    }
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("plusIndent", plusIndent).toString();
    }
}
