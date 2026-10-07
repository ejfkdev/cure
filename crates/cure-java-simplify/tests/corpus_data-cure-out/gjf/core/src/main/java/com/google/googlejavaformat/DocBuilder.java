package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;
import java.util.ArrayDeque;
import java.util.List;

public final class DocBuilder {
    private final Doc.Level base = Doc.Level.make(Indent.Const.ZERO);
    private final ArrayDeque<Doc.Level> stack = new ArrayDeque<>();
    private Doc.Level appendLevel = base;
    public DocBuilder() {
        stack.addLast(base);
    }
    public DocBuilder withOps(List<Op> ops) {
        for (Op op : ops) {
            op.add(this);
        }
        return this;
    }
    void open(Indent plusIndent) {
        Doc.Level level = Doc.Level.make(plusIndent);
        stack.addLast(level);
    }
    void close() {
        Doc.Level top = stack.removeLast();
        stack.peekLast().add(top);
    }
    void add(Doc doc) {
        appendLevel.add(doc);
    }
    void breakDoc(Doc.Break breakDoc) {
        appendLevel = stack.peekLast();
        appendLevel.add(breakDoc);
    }
    public Doc build() {
        return base;
    }
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("base", base).add("stack", stack).add("appendLevel", appendLevel).toString();
    }
}
