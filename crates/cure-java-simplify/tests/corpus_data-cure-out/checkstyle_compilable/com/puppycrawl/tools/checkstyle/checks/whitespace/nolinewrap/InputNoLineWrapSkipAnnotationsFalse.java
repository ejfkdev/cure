package com.puppycrawl.tools.checkstyle.checks.whitespace.nolinewrap;

@SuppressWarnings("unused")
public class InputNoLineWrapSkipAnnotationsFalse {
    @Deprecated
    public InputNoLineWrapSkipAnnotationsFalse() {}
    @SafeVarargs
    public final <T> void foo(T... elements) {}
    @Deprecated
    public enum InputNoLineWrapSkipAnnotationsFalseEnum {
        FOO
    }
    @Deprecated
    public record InputNoLineWrapSkipAnnotationsFalseRecord(String foo) {
        @Deprecated
        public InputNoLineWrapSkipAnnotationsFalseRecord {}
    }
}
