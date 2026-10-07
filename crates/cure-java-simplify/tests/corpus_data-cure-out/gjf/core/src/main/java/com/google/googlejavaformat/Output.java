package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;
import com.google.common.collect.Range;
import com.google.googlejavaformat.OpsBuilder.BlankLineWanted;
import java.util.Optional;

public abstract class Output extends InputOutput {
    public static final class BreakTag {
        Optional<Boolean> taken = Optional.empty();
        public void recordBroken(boolean broken) {
            taken = Optional.of(broken);
        }
        public boolean wasBreakTaken() {
            return taken.orElse(false);
        }
    }
    public abstract void indent(int indent);
    public abstract void append(String text, Range<Integer> range);
    public abstract void blankLine(int k, BlankLineWanted wanted);
    public abstract void markForPartialFormat(Input.Token start, Input.Token end);
    public abstract CommentsHelper getCommentsHelper();
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("super", super.toString()).toString();
    }
}
