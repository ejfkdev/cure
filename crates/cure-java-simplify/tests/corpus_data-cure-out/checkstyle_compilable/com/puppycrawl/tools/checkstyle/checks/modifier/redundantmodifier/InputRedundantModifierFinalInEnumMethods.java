package com.puppycrawl.tools.checkstyle.checks.modifier.redundantmodifier;

public enum InputRedundantModifierFinalInEnumMethods {
    E1, E2 {
        @Override
        public final void v() { // violation 'Redundant 'final' modifier.'
        }
    };
    public void v() {}
    public final void v2() {}
}

enum InputRedundantModifierFinalInEnumMethods2 {
    E1 {
        @Override
        public final void v() { // violation 'Redundant 'final' modifier.'
        }
    }, E2 {
        @Override
        public void v() {
        }
    };
    public abstract void v();
}
