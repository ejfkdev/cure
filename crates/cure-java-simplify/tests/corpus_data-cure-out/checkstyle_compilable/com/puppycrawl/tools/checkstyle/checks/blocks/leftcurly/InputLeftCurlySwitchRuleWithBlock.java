package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

public class InputLeftCurlySwitchRuleWithBlock {
    void test(int x) {
        int result = switch (x) {
            case 1 -> {
                yield 1;
            }
            default -> 0;
        };
    }
}
