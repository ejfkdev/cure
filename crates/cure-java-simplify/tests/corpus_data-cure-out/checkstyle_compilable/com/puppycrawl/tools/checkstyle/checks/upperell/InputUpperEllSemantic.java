package com.puppycrawl.tools.checkstyle.checks.upperell;

class InputUpperEllSemantic {
    static {
        Boolean x = new Boolean(true);
    }
    static {
        int a = 0;
    }
    static {}
    private static final long IGNORE = 666l + 666L;
    public void triggerEmptyBlockWithoutBlock() {}
    synchronized void foo() {
        synchronized (this) {}
        synchronized (Class.class) {
            synchronized (new Object()) {}
        }
    }
}
