package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionKeywordsAndOperators {
    private int mVar1 = 1;
    private int mVar2 = 1;
    private int mVar3 = 1;
    void method1() {
        int b = 2;
        b -= -1 + b;
        b = b++ + b--;
        b = ++b - --b;
    }
    void method2() {
        synchronized (this) {}
        try {} catch (RuntimeException e) {}
    }
    private int mVar4 = 1;
    private void fastExit() {}
    private int nonVoid() {
        return 2;
    }
    private void testCasts() {
        Object o = (Object) new Object();
    }
    private void testQuestions() {}
    private void starTest() {}
    private void boolTest() {}
    private void divTest() {}
    private java.lang.String dotTest() {
        Object o = new java.lang.Object();
        o.toString();
        o.toString();
        o.toString();
        return o.toString();
    }
    public void assertTest() {
        assert true;
        assert true : "Whups";
        assert "OK".equals(null) ? false : true : "Whups";
        assert true;
        assert true : "Whups";
    }
    void donBradman(Runnable aRun) {
        donBradman(new Runnable() {
            public void run() {
            }
        });
        Runnable r = new Runnable() {
            public void run() {
            }
        };
    }
    void rfe521323() {
        doStuff();
        for (int i = 0; i < 5; i++) {}
    }
    private int i;
    private int i1, i2, i3;
    private int i4, i5, i6;
    void bug806243() {
        Object o = new InputAntlr4AstRegressionKeywordsAndOperators() {
            private int j ;
            //           ^ whitespace
        };
    }
    void doStuff() {}
}

interface IFoo {
    void foo();
}

class SpecialCasesInForLoop {
    void forIterator() {
        for (int i = 0; i++ < 5; ) {}
        int i = 0;
        for (; i < 5; i++) {}
        for (int anInt : getSomeInts()) {}
    }
    int[] getSomeInts() {
        return null;
    }
    public void myMethod() {
        new Thread() {
            public void run() {
            }
        }.start();
    }
    public void foo(java.util.List<? extends String[]> bar, Comparable<? super Object[]> baz) {}
    public void mySuperMethod() {
        new Runnable() {
                public void run() {
                }
            }.run();
    }
    public void testNullSemi() {}
    public void register(Object obj) {}
    public void doSomething(String[] args) {
        register(boolean[].class);
        register(args);
    }
    public void parentheses() {
        testNullSemi();
    }
    public static void testNoWhitespaceBeforeEllipses(String... args) {}
    public String test() {
        return "00000";
    }
}
