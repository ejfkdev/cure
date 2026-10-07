package com.puppycrawl.tools.checkstyle.checks.whitespace.emptylineseparator;

import java.lang.Object;
import java.lang.Class;
import java.lang.Integer;
import java.lang.Long;
import java.lang.Float;
import java.lang.Double;
import java.lang.Character;
import java.lang.String;
import java.lang.Object;
import java.lang.Boolean;
import java.lang.Byte;
import java.lang.Short;
import java.lang.Number;
import java.lang.Runnable;
import java.lang.Thread;
import java.lang.StringBuilder;
import static java.lang.Math.abs;

public class InputEmptyLineSeparatorWithComments {
    public int testViolationWithoutComment = 1;
    public int testNoViolationWithoutComment = 2;
    public int testViolationWithSingleLineComment = 3;
    public int testNoViolationWithSingleLineComment = 4;
    public int testViolationWithMultilineComment = 5;
    public int testNoViolationWithMultilineComment = 6;
    public int testViolationWithJavadoc = 7;
    public int testNoViolationWithJavadoc = 8;
    public void testViolationWithoutComment() {}
    public void testNoViolationWithoutComment() {}
    public void testViolationWithSingleLineComment() {}
    public void testNoViolationWithSingleLineComment() {}
    public void testViolationWithMultilineComment() {}
    public void testNoViolationWithMultilineComment() {}
    public void testViolationWithJavadoc() {}
    public void testNoViolationWithJavadoc() {}
    public static class Class1 {
    }
    public static class Class2 {
    }
    public static class Class3 {
    }
    public static class Class4 {
    }
    public
    // ok, because no more than 1 empty lines before
    static class Class5 {
    }
    public
    /* something */
    static class Class6 {
    }
    public static class Class7 {
    }
    public static class Class8 {
    }
    public static class Class9 {
    }
    public static class Class10 {
        {}
    }
    public interface Interface1 {
    }
    public interface Interface2 {
    }
    public interface Interface3 {
    }
    interface Interface4 {
    }
    interface Interface5 {
    }
    public enum Enum1 {
        E1, E2
    }
    public enum Enum2 {
    }
    public enum Enum3 {
    }
    public enum Enum4 {
    }
    public enum Enum5 {
    }
    public


    // violation ''//' has more than 1 empty lines before.'
    static enum Enum6 {
    }
    static {
        abs(2);
    }
    {
        abs(1);
    }
    {}
    {}
    public InputEmptyLineSeparatorWithComments() {
        testNoViolationWithJavadoc = 1;
    }
    public InputEmptyLineSeparatorWithComments(int i) {
        testNoViolationWithJavadoc = 1;
    }
    public InputEmptyLineSeparatorWithComments(int i, int j) {
        testNoViolationWithJavadoc = 1;
    }
    public InputEmptyLineSeparatorWithComments(int i, int j, int k) {
        testNoViolationWithJavadoc = 1;
    }
}
