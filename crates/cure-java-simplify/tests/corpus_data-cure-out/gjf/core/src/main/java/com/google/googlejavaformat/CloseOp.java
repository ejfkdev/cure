package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;

public enum CloseOp implements Op {
    CLOSE;
    public static Op make() {
        return CLOSE;
    }
    @Override
  public void add(DocBuilder builder) {
        builder.close();
    }
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).toString();
    }
}
