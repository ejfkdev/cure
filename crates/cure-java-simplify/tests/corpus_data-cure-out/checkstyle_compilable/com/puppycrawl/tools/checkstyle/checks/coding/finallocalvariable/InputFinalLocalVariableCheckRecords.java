package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public record InputFinalLocalVariableCheckRecords(boolean t, boolean f) {
    public InputFinalLocalVariableCheckRecords {}
    record bad(int i) {
        public bad {}
    }
}
