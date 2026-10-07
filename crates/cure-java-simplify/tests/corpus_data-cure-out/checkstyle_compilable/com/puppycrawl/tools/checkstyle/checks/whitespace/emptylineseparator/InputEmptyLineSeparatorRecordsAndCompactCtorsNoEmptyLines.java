package com.puppycrawl.tools.checkstyle.checks.whitespace.emptylineseparator;

public class InputEmptyLineSeparatorRecordsAndCompactCtorsNoEmptyLines {
    public void foo() {}
    public record MyRecord1() {
        public MyRecord1 {}
    }
}
