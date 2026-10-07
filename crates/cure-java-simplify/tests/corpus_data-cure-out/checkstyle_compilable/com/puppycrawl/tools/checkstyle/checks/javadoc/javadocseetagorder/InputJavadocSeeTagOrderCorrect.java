package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocseetagorder;

import java.util.List;

public class InputJavadocSeeTagOrderCorrect {
    private String field;
    public InputJavadocSeeTagOrderCorrect() {}
    public InputJavadocSeeTagOrderCorrect(String value) {}
    private void getName() {}
    private void getName(String value) {}
    static class OtherClass {
        private int field;
        OtherClass() {}
        OtherClass(String value) {}
        private void getName() {}
        private void getName(String value) {}
    }
}
