package com.puppycrawl.tools.checkstyle.checks.annotation.suppresswarnings;

import java.lang.annotation.Documented;

@SuppressWarnings(value={"unchecked", "unused"})
public class InputSuppressWarningsExpandedNonConstant3 {
    @SuppressWarnings(value={"   "}) class Empty {
        @SuppressWarnings(value={"unchecked", ""})
        public Empty() {}
    }
    @SuppressWarnings(value={"unused"}) enum Duh {
        @SuppressWarnings(value={"unforgiven", "    un"})
        D;
        public static void foo() {
            @SuppressWarnings(value={"unused"})
                        Object o = new InputSuppressWarningsExpandedNonConstant3() {

                            @Override
                            @SuppressWarnings(value={"unchecked"})
                            // violation above 'The warning 'unchecked' cannot be suppressed at this location'
                            public String toString() {
                                return "";
                            }
                        };
        }
    }
    @SuppressWarnings(value={"invalid"})
    @Documented @interface Sweet {
        int cool();
    }
    @Documented
    @SuppressWarnings(value={}) @interface MoreSweetness {
        @SuppressWarnings(value={"unused", "ignore"})
                int cool();
    }
    public class Junk {
        @SuppressWarnings(value={}) int a = 1;
        @SuppressWarnings(value={"unchecked"})
        @Deprecated int b = 1;
        void doFoo(String s, @SuppressWarnings(value={"unchecked"}) String y) {}
    }
    @SuppressWarnings(value={(false) ? "unchecked" : "", (false) ? "unchecked" : ""}) class Cond {
        @SuppressWarnings(value={(false) ? "" : "unchecked"})
        public Cond() {}
        @SuppressWarnings(value={(false) ? (true) ? "   " : "unused" : "unchecked",
            (false) ? (true) ? "   " : "unused" : "unchecked"})
        // violation above 'The warning 'unchecked' cannot be suppressed at this location'
        public void aCond1() {}
        @SuppressWarnings(value={(false) ? "unchecked" : (true) ? "   " : "unused"})
        public void aCond2() {}
        @java.lang.SuppressWarnings(value={(false) ? "unchecked" :
                    ("" == "") ? (false) ? (true) ? "" : "foo" : "   " : "unused",
                (false) ? "unchecked" :
                // violation above 'The warning 'unchecked' cannot be suppressed at this location'
                    ("" == "") ? (false) ? (true) ? "" : "foo" : "   " : "unused"})
        public void seriously() {}
    }
}
