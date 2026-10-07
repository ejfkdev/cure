package com.puppycrawl.tools.checkstyle.checks.javadoc.summaryjavadoc;

public class InputSummaryJavadocInlineForbidden2 {
    InputSummaryJavadocInlineForbidden2 anon = new InputSummaryJavadocInlineForbidden2() {
                // violation below 'First sentence .* missing an ending period.'
                /**
                 * mm{@inheritDoc}
                 */
                void foo7() {
                }

                /**
                 * {@summary {@code see}.}
                 */
                void foo10() {
                }
            };
    public void validInlineJavadoc() {}
    void foo12() {}
    public class TestClass {
    }
    public int validInlineJavadocReturn(int a) {
        return a;
    }
}
