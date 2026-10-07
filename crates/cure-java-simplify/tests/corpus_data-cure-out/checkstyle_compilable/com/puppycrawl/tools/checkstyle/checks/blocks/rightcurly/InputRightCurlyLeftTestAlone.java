package com.puppycrawl.tools.checkstyle.checks.blocks.rightcurly;

class InputRightCurlyLeftTestAlone {
    int foo() throws InterruptedException {
        int x = 1;
        while (true) {
            try {
                if (x > 0) {
                    break;
                } else if (x >= 0) {
                    break;
                }
                switch (2) {
                    case 0:
                        break;
                    default:
                        break;
                }
            } catch (Exception e) {
                break;
            } finally {
                break;
            }
        }
        synchronized (this) {
            do {
                x = 2;
            } while (x == 2);
        }
        this.wait(666);
        for (int k = 0; k < 1; k++) {}
        return System.currentTimeMillis() > 1000 ? 1 : 2;
    }
    static {}
    public enum GreetingsEnum {
        HELLO, GOODBYE
    }
    void method2() {
        boolean flag = true;
        if (flag) {
            System.identityHashCode("heh");
            flag = !flag;
        }
        String.CASE_INSENSITIVE_ORDER.equals("Xe-xe");
    }
}

class FooCtorTestAlone {
    int i;
    public void FooCtor() {
        i = 1;
    }
}

class FooMethodTestAlone {
    public void fooMethod() {}
}

class FooInnerTestAlone {
    class InnerFoo {
        public void fooInnerMethod() {}
    }
}

class Absent_CustomFieldSerializer3TestAlone {
    public static void serialize() {}
}

class Absent_CustomFieldSerializer4TestAlone {
    public void Absent_CustomFieldSerializer4() {}
}

class EmptyClass2TestAlone {
}

interface EmptyInterface3TestAlone {
}

class ClassWithStaticInitializersTestAlone {
    static {}
    static {}
    static class Inner {
        static {}
    }
    public void emptyBlocks() {
        try {} catch (RuntimeException e) {
            new Object();
        } catch (Exception e) {} catch (Throwable e) {}
        do {} while (true);
    }
    public void codeAfterLastRightCurly() {
        while (new Object().equals(new Object())) {}
        for (int i = 0; i < 1; i++) {
            new Object();
        }
    }
    static final java.util.concurrent.ThreadFactory threadFactory = new java.util.concurrent.ThreadFactory() {
        @Override
        public Thread newThread(final Runnable r) {
            return new Thread(r);
        }};
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
}
