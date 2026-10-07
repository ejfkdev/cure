package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;
import com.google.googlejavaformat.Output.BreakTag;

public abstract class Indent {
    abstract int eval();
    public static final class Const extends Indent {
        private final int n;
        public static final Const ZERO = new Const(0);
        private Const(int n) {
            this.n = n;
        }
        public static Const make(int n, int indentMultiplier) {
            return new Const(n * indentMultiplier);
        }
        @Override int eval() {
            return n;
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("n", n).toString();
        }
    }
    public static final class If extends Indent {
        private final BreakTag condition;
        private final Indent thenIndent;
        private final Indent elseIndent;
        private If(BreakTag condition, Indent thenIndent, Indent elseIndent) {
            this.condition = condition;
            this.thenIndent = thenIndent;
            this.elseIndent = elseIndent;
        }
        public static If make(BreakTag condition, Indent thenIndent, Indent elseIndent) {
            return new If(condition, thenIndent, elseIndent);
        }
        @Override int eval() {
            return (condition.wasBreakTaken() ? thenIndent : elseIndent).eval();
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("condition", condition).add("thenIndent", thenIndent).add("elseIndent", elseIndent).toString();
        }
    }
}
