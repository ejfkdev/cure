package com.puppycrawl.tools.checkstyle.checks.coding.requirethis;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class InputRequireThisEnumInnerClassesAndBugs3 {
    int i;
    void method1() {
        i = 3;
    }
    void method2(int i) {
        i++;
        this.i = i;
        method1();
        try {
            this.method1();
        } catch (RuntimeException e) {
            e.toString();
        }
        this.i--;
        "10";
    }
    <T> void method3() {
        i = 3;
    }
    void method4() {
        this.method3();
        this.method3();
    }
    int I = 0;
    private class I {
    }
}

enum MyEnum3 {
    A, B
    {
        void doSomething()
        {
            z = 1; // violation 'Reference to instance variable 'z' needs "this.".'
        }
    };
    int z;
    private MyEnum3() {
        z = 0;
    }
}

class Bug21230033 {
    @Rock(band = "GnR")
    private String band;
    class Inner {
        @Rock(band = {"GnR"})
        private String band;
    }
    class Inner2 {
        @Rock(band = {"Tool"})
        private String band;
    }
    @interface Rock {
        String[] band() default "Metallica";
    }
}

class Bug11559213 {
    private static int CONST = 1;
    private static int static_method() {
        return 1;
    }
    private int method1() {
        return CONST;
    }
    private int method2() {
        return static_method();
    }
}

interface Issue1553 {
    String BASE = "A";
    String EXT = BASE + "B";
}

class Issue2573 {
    public void foo() {
        try (InputStream foo = new ByteArrayInputStream(new byte[512])) {
            foo.read();
        } catch (IOException e) {
            e.getCause();
        }
    }
}

class Issue22403 {
    int i;
    void foo() {
        i++;
        i++;
        int i = 1;
        i++;
        instanceMethod();
    }
    void instanceMethod() {}
    class Nested {
        void bar() {
            instanceMethod();
            i++;
        }
    }
}

class Issue25393 {
    void foo(int i) {}
    static void foo(double i) {}
    void foo() {}
    void bar() {
        foo(1);
        foo();
    }
}

class NestedRechange3 {
    final String s = "";
    NestedRechange3() {
        String s = "t";
        s = s.substring(0);
    }
    private static class NestedStatic {
        static final String s = "";
        public void method() {
            s.substring(0);
        }
    }
}

class NestedFrames3 {
    int a = 0;
    int b = 0;
    public int oneReturnInMethod2() {
        for (int i = 0; i < 10; i++) {}
        return a + a * a;
    }
    public int oneReturnInMethod3() {
        for (int b = 0; b < 10; b++) {}
        return b + b * b;
    }
    final NestedFrames NestedFrames = new NestedFrames();
}
