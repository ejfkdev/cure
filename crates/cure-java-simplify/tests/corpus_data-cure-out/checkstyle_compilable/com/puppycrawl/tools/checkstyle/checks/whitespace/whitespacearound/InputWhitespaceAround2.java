package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

@SuppressWarnings({"this", "that"})
public class InputWhitespaceAround2 {
    protected InputWhitespaceAround2(int i) {
        this();
        toString();
    }
    protected InputWhitespaceAround2() {
        super();
    }
    public void enhancedFor() {
        int[] i = new int[2];
        for (int j : i) {
            System.identityHashCode(j);
        }
    }
}

@interface CronExpression2 {
    Class<?>[] groups() default {};
}

@interface CronExpression12 {
    Class<?>[] groups() default { }; // extra space
}
