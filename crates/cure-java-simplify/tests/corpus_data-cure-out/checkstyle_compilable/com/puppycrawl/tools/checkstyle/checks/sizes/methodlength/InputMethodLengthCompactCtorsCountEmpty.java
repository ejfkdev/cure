package com.puppycrawl.tools.checkstyle.checks.sizes.methodlength;

public class InputMethodLengthCompactCtorsCountEmpty {
    record MyTestRecord() {
        static {
            System.out.println("test");
            System.out.println("test");
            System.out.println("test");
        }
    }
    record MyTestRecord2() {
        public MyTestRecord2 {
            System.out.println("test");
        }
    }
    record MyTestRecord3(String str) {
        void foo() {
            System.out.println("test");
        }
    }
    record MyTestRecord4(int x, int y) {
        public MyTestRecord4() {
            this(4, 5);
        }
    }
    record MyTestRecord5() {
        public MyTestRecord5() {}
    }
    class LocalRecordHelper {
        Class<?> m(int x) {
            record R76(int x) {

                            public R76 { // violation 'Method .* length is 8 lines (max allowed is 2).'
                                int y = 5;
                                int z = 10;
                                String newString = String.valueOf(y);
                                System.out.println(newString);
                                System.out.println("Value of y: " + newString);
                                System.out.println(y + z);
                            }

                        }
                        return R76.class;
        }
    }
}
