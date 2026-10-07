package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

public class InputWhitespaceAroundRecordsAllowEmptyTypes {
    record MyRecord() {
    }
    record MyRecord1() {
    }
    record MyRecord2() {
        class MyClass {
        }
        interface Foo {
        }
        record MyRecord() {
        }
    }
}
