package com.puppycrawl.tools.checkstyle.checks.descendanttoken;

public class InputDescendantTokenReturnFromFinally5 {
    public void foo() {
        try {
            System.currentTimeMillis();
        } finally {
            return;
        }
    }
    public void bar() {
        try {
            System.currentTimeMillis();
        } finally {
            if (System.currentTimeMillis() == 0) {
                return;
            }
        }
    }
    public void thisNull() {
        boolean result3 = this.getClass().getName() == String.valueOf(null == System.getProperty("abc"));
    }
}
