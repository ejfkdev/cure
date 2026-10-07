package com.puppycrawl.tools.checkstyle.checks.whitespace.singlespaceseparator;

public class InputSingleSpaceSeparatorReservedWords {
    void testIf(int x) {
        if (x > 0) {
            System.out.println("Positive");
        } else {
            return;
        }
    }
    void testFor() {
        for (int i = 0; i < 5; i++) {
            System.out.println(i);
        }
    }
}
