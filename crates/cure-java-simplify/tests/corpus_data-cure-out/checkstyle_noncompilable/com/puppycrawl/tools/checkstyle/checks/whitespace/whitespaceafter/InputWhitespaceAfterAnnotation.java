package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespaceafter;

import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.util.List;

public class InputWhitespaceAfterAnnotation {
    @Target(ElementType.TYPE_USE) @interface Size {
        int max();
    }
    @Target(ElementType.TYPE_USE) @interface AnnoType {
    }
    @Target(ElementType.TYPE_USE) @interface NonNull2 {
    }
    @Target(ElementType.TYPE_USE) @interface NonNull {
    }
    @NonNull int[][] field1;
    @NonNull int[][] field2;
    void test1(String... param) {}
    void test2(String... param) {}
    void test3(String[]... param) {}
    void test4(String[]... param) {}
    void test5(String[]... param) {}
    void test6(String[]... param) {}
    void test7(String... names) {}
    void test8(String... names) {}
    public String[][] test9() {
        return null;
    }
    public String[][] test10() {
        return null;
    }
    public void test11(final char[] a) {}
    public void test12(final char[] a) {}
    public void test13(final char[] a) {}
    public void test14(final char[] a) {}
    public @AnnotationAfterTest String[] test15() {
        return;
        @NonNull2 String @AnnotationAfterTest[3];
                // violation above ''AnnotationAfterTest' is not followed by whitespace'
    }
    public record Example(char @AnnotationAfterTest[] data) {
    }
    public record Example1(
        List<@NonNull String> names,
        @NonNull String value,
        String @AnnotationAfterTest[] array
        // violation above ''AnnotationAfterTest' is not followed by whitespace'
    ) {
    }
    class ACls {
    }
    .Type AKls<@Tacos.Type String>.@Tacos.Type BClas<@Tacos.Type Number> {

            public ACls(final AKls<String> enclosingInstance) {
                enclosingInstance.super();
            }
        }

        class AKls<T> {
            class BClas<U> {}
        }

        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER})
        @interface AnnotationAfterTest {}
}

class Tacos {
    @Target({ElementType.TYPE_USE})
    public @interface Type {
    }
}
