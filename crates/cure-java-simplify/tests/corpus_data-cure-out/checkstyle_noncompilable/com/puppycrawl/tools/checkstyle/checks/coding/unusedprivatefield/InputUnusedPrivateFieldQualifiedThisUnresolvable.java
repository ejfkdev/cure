package com.puppycrawl.tools.checkstyle.checks.coding.unusedprivatefield;

public class InputUnusedPrivateFieldQualifiedThisUnresolvable {
    class Inner {
        private int field;
        void method() {
            int x = NotAnEnclosingClass.this.field;
        }
    }
}
