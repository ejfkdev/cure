package com.puppycrawl.tools.checkstyle.checks.coding.nofinalizer;

public class InputNoFinalizerHasFinalizer {
    public void finalize() {
        new Runnable() {

            public void run() {
                reallyFinalize("hi");
            }

            // generates a PARAMETER_DEF AST inside the METHOD_DEF of finalize()
            private void reallyFinalize(String s)
            {
            }
        }.run();
    }
    public void finalize(String x) {}
}
