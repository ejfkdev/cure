package com.puppycrawl.tools.checkstyle.checks.sizes.executablestatementcount;

public class InputExecutableStatementCountDefaultConfig {
    public void foo() {
        while (true) {
            new Thread(new Runnable() {
                public void run() {
                    while (true) {
                    }
                }
            }).start();
        }
    }
    public void bar() {
        if (System.currentTimeMillis() == 0) {
            if (System.currentTimeMillis() == 0 && System.currentTimeMillis() == 0) {}
            if (System.currentTimeMillis() == 0 || System.currentTimeMillis() == 0) {}
        }
    }
    public void simpleElseIf() {
        if (System.currentTimeMillis() != 0) 
            if (System.currentTimeMillis() == 0) {}
    }
    public void stupidElseIf() {
        if (System.currentTimeMillis() != 0) {
            if (System.currentTimeMillis() != 0) {
                if (System.currentTimeMillis() == 0) {}
            }
            if (System.currentTimeMillis() == 0) {}
        }
    }
    public InputExecutableStatementCountDefaultConfig() {
        if (System.currentTimeMillis() != 0) 
            if (System.currentTimeMillis() == 0) {}
    }
    static {
        if (System.currentTimeMillis() != 0) 
            if (System.currentTimeMillis() == 0) {}
    }
    {
        if (System.currentTimeMillis() != 0) 
            if (System.currentTimeMillis() == 0) {}
    }
    public InputExecutableStatementCountDefaultConfig(int aParam) {
        new Thread(new Runnable() {
            public void run() {
                while (true) {
                }
            }
        }).start();
    }
    public InputExecutableStatementCountDefaultConfig(String someString) {}
    static Runnable r1 = () -> {};
}
