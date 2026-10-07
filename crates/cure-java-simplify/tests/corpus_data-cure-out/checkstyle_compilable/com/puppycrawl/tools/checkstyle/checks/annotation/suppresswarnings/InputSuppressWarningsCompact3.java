package com.puppycrawl.tools.checkstyle.checks.annotation.suppresswarnings;

import java.lang.annotation.Documented;

@SuppressWarnings({"unchecked", "unused"})
public class InputSuppressWarningsCompact3 {
    @SuppressWarnings({"   "}) class Empty {
        @SuppressWarnings({"unchecked", ""})
        public Empty() {}
    }
    @SuppressWarnings({"unused"}) enum Duh {
        @SuppressWarnings({"unforgiven", "    un"})
        D;
        public static void foo() {
            @SuppressWarnings({"unused"})
                        Object o = new InputSuppressWarningsCompact3() {

                            @Override
                            @SuppressWarnings({"unchecked"})
                            // violation above 'The warning 'unchecked' cannot be suppressed at this location'
                            public String toString() {
                                return "";
                            }
                        };
        }
    }
    @SuppressWarnings({"invalid"})
    @Documented @interface Sweet {
        int cool();
    }
    @Documented
    @SuppressWarnings({}) @interface MoreSweetness {
        @SuppressWarnings({"unused", "ignore"})
                int cool();
    }
    public class Junk {
        @SuppressWarnings({}) int a = 1;
        @SuppressWarnings({"unchecked"})
        @Deprecated int b = 1;
        void doFoo(String s, @SuppressWarnings({"unchecked"}) String y) {}
    }
}
