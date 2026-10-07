package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocutilizingtrailingspace;

public class InputJavadocUtilizingTrailingSpaceBlockTags {
    public int methodWithTags(int first, int second) {
        if (first < 0 || second < 0) {
            throw new IllegalArgumentException("Negative");
        }
        return first + second;
    }
    public void tagWithIndentedValue(int value) {}
    public void multipleParams(int a, int b) {}
    public int returnIndented() {
        return 0;
    }
    public void throwsMultiLine() {}
    public void multipleSee() {}
    @Deprecated
    public void oldMethod() {}
    public void newMethod() {}
}
