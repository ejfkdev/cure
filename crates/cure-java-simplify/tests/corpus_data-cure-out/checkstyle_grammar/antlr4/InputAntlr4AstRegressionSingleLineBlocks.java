package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionSingleLineBlocks {
    public void testMethod() {}
    public void testMethod1() {}
    public class TestClass {
    }
    public class TestClass1 {
    }
    public class TestClass2 {
        public TestClass2() {}
        public TestClass2(String someValue) {}
    }
    public void testMethod11() {}
    public @interface TestAnnotation5 {
        String someValue();
    }
    public @interface TestAnnotation6 {
    }
    public @interface TestAnnotation7 {
        String someValue();
    }
    public @interface TestAnnotation8 {
        String someValue();
    }
    public @interface TestAnnotation9 {
        String someValue();
    }
    enum TestEnum {
    }
    enum TestEnum1 {
        SOME_VALUE
    }
    enum TestEnum2 {
        SOME_VALUE
    }
    enum TestEnum3 {
        SOME_VALUE
    }
    enum TestEnum4 {
        SOME_VALUE
    }
    interface Interface1 {
        int i = 1;
        public void meth1();
    }
    interface Interface2 {
        int i = 1;
        public void meth1();
    }
    interface Interface3 {
        void display();
        interface Interface4 {
            void myMethod();
        }
    }
    interface InterfaceEndingWithSemiColon2 {
        public void fooEmpty();
    }
}
