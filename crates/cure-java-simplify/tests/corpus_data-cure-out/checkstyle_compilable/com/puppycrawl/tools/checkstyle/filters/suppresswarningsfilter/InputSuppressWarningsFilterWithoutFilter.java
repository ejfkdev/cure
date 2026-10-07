package com.puppycrawl.tools.checkstyle.filters.suppresswarningsfilter;

@SuppressWarnings("foo") class InputSuppressWarningsFilterWithoutFilter {
    @SuppressWarnings("foo") interface I {
    }
    @SuppressWarnings("foo") enum E {
    }
    @SuppressWarnings("foo") InputSuppressWarningsFilterWithoutFilter() {}
    @SuppressWarnings("foo") @interface A {
    }
    @SuppressWarnings("unused") int I;
    @SuppressWarnings({"membername"})
    private int J;
    private int K;
    @SuppressWarnings(value="membername")
    private int L;
    private int X;
    @SuppressWarnings("checkstyle:ConstantName")
    private static final int m = 0;
    private static final int n = 0;
    @SuppressWarnings("paramnum") void foo(@SuppressWarnings("unused") int a, int b, int c, int d, int e, int f, int g, int h) {
        @SuppressWarnings("unused") int z;
        try {} catch (Exception ex) {}
    }
    @java.lang.SuppressWarnings("illegalCatch")
    public void needsToCatchException() {
        try {} catch (Exception ex) {}
    }
    enum AnEnum {
        @SuppressWarnings("rawtypes")
        ELEMENT
    }
    private static final String UNUSED = "UnusedDeclaration";
    @SuppressWarnings(UNUSED)
    public void annotationUsingStringConstantValue() {}
    @SuppressWarnings("checkstyle:uncommentedmain")
    public static void main(String[] args) {}
    static class TestClass1 {
        @SuppressWarnings("uncommentedmain")
        public static void main(String[] args) {}
    }
    static class TestClass2 {
        @SuppressWarnings("UncommentedMain")
        public static void main(String[] args) {}
    }
    static class TestClass3 {
        @SuppressWarnings("checkstyle:UncommentedMain")
        public static void main(String[] args) {}
    }
    @SuppressWarnings("checkstyle:javadoctype")
    public static abstract class Task {
    }
}
