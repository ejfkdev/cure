package com.puppycrawl.tools.checkstyle.checks.metrics.javancss;

public class InputJavaNCSSRecordsAndCompactCtors {
    class TestClass {
        private void testMethod1() {
            int y = 2;
        }
        private void testMethod2() {
            int abc = 1;
        }
    }
    record MyRecord1(boolean t, boolean f) {
        public MyRecord1 {
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
        }
    }
    record MyRecord2(boolean a, boolean b) {
        MyRecord2() {
            this(true, false);
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
        }
        private void testMethod() {
            for (int i = 0; i < 10; i++) {
                if (i == 0) {
                    int y = 2;
                } else {
                    int abc = 1;
                }
            }
        }
    }
    record MyRecord3(boolean a, boolean b) {
        public void foo() {
            record TestInnerRecord() {

                    private static Object test;
                  }
                  System.out.println("test");
            System.out.println("test");
        }
    }
    record MyRecord4(int x, int y) {
        record TestInnerRecord() {
            private static Object test;
        }
    }
    record MyRecord5(int x, int y) {
        public MyRecord5 {
            if (x > 5) {
                System.out.println("x greater than 5!");
            }
        }
    }
    record MyRecord6(int x, int y) {
        public MyRecord6 {}
    }
    record MyRecord7(int x, int y) {
        public MyRecord7 {
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
        }
    }
}
