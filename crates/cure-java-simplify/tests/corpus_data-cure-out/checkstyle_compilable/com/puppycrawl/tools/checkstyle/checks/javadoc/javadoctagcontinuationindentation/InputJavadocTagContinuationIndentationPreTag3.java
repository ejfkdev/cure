package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctagcontinuationindentation;

import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class InputJavadocTagContinuationIndentationPreTag3 {
    @Target(ElementType.TYPE_USE) @interface Internal {
    }
    @Internal
    public Object testMethod() {
        return new Object();
    }
    @Internal
    public Object testMethod2() {
        return new Object();
    }
}
