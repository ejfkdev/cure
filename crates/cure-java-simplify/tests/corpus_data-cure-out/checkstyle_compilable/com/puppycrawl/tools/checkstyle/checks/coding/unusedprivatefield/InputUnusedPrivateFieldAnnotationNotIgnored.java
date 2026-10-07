package com.puppycrawl.tools.checkstyle.checks.coding.unusedprivatefield;

public class InputUnusedPrivateFieldAnnotationNotIgnored {
    @interface Inject {
    }
    @Inject
    private Object service;
    private int unused;
}
