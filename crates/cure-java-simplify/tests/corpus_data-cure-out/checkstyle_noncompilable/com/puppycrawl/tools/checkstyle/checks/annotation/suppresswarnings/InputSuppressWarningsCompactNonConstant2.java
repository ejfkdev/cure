package com.puppycrawl.tools.checkstyle.checks.annotation.suppresswarnings;

import java.lang.annotation.Documented;

@SuppressWarnings({"unchecked", "unused"})
public class InputSuppressWarningsCompactNonConstant2 {
    @SuppressWarnings({"   "}) class Empty {
        @SuppressWarnings({"unchecked", ""})
        public Empty() {}
    }
    @SuppressWarnings({"unused"}) enum Duh {
        @SuppressWarnings({"unforgiven", "    un"})
        D;
        public static void foo() {
            @SuppressWarnings({"unused"})
                        Object o = new InputSuppressWarningsCompactNonConstant2() {

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
    @SuppressWarnings({(false) ? "unchecked" : "", (false) ? "unchecked" : ""}) class Cond {
        @SuppressWarnings({(false) ? "" : "unchecked"})
        public Cond() {}
        @SuppressWarnings({(false) ? (true) ? "   " : "unused" : "unchecked",
            (false) ? (true) ? "   " : "unused" : "unchecked"})
        public void aCond1() {}
        @SuppressWarnings({(false) ? "unchecked" : (true) ? "   " : "unused"})
        public void aCond2() {}
        @java.lang.SuppressWarnings({(false) ? "unchecked" :
                ("" == "") ? (false) ? (true) ? "" : "foo" : "   " : "unused",
            (false) ? "unchecked" : ("" == "") ? (false) ? (true) ? "" :
                    "foo" : "   " :
                    "unused"})
        // 2 violations 3 lines above:
        // 'cannot be suppressed at this location'
        // 'cannot be suppressed at this location'
        // 2 violations 5 lines above:
        // 'cannot be suppressed at this location'
        // 'cannot be suppressed at this location'
        // violation 7 lines above 'The warning 'unused' cannot be suppressed at this location'
        public void seriously() {}
    }
}
