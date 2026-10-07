package com.puppycrawl.tools.checkstyle.checks.annotation.suppresswarnings;

import java.lang.annotation.Documented;

@SuppressWarnings(value = {"unchecked", "unused"})
public record InputSuppressWarningsRecords(String x) {
    @SuppressWarnings(value = {"unchecked", ""})
    public InputSuppressWarningsRecords {}
    @SuppressWarnings(value = {"   "}) class Empty {
        @SuppressWarnings(value = {"unchecked", ""})
        public Empty() {}
    }
    @SuppressWarnings(value = {"unused"}) enum Duh {
        @SuppressWarnings(value = {"unforgiven", "    un"})
        D;
        public static void foo() {
            Object myHashMap;
            @SuppressWarnings(value = {"unused"}) int x = 42;
        }
    }
    @SuppressWarnings(value = {"invalid"})
    @Documented @interface inter {
        int cool();
    }
    @Documented
    @SuppressWarnings(value = {}) @interface MoreSweetness {
        @SuppressWarnings(value = {"unused", "something else"})
                int cool();
    }
    public record MyRecord() {
        @SuppressWarnings(value = {})
        static int a = 1;
        @SuppressWarnings(value = {"unchecked"})
        @Deprecated
        static int b = 1;
        void doFoo(String s, @SuppressWarnings(value = {"unchecked"}) String y) {}
    }
}
