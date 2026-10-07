package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctype;

class InputJavadocTypeWhitespace {
    void donBradman(Runnable aRun) {
        donBradman(new Runnable() {
            public void run() {
            }
        });
        Runnable r = new Runnable() {
            public void run() {
            }
        };
    }
    void bug806243() {
        Object o = new InputJavadocTypeWhitespace() {
            private int j ;
        };
    }
}

interface IFoo {
    void foo();
}

class SpecialCasesInForLoop {
    public void myMethod() {
        new Thread() {
            public void run() {
            }
        }.start();
    }
}
