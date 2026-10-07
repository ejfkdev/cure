package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public record InputFinalLocalVariableCheckRecords(boolean t, boolean f) {
    public InputFinalLocalVariableCheckRecords {
        int a = 1;
    }
    record bad(int i) {
        public bad {
            int b = 0;
        }
    }
}
