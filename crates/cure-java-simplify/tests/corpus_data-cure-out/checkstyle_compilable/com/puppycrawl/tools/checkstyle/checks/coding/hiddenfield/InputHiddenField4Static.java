package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

class InputHiddenField4Static {
    private static int hidden;
    public static void staticMethod() {
        int hidden;
    }
    public void method() {
        int hidden;
    }
    static {
        int hidden;
    }
    {
        int hidden;
    }
}

class StaticMethods4 {
    private int notHidden;
    public static void method() {
        int notHidden;
    }
    static {
        int notHidden;
    }
    private int x;
    private static int y;
    static class Inner {
        void useX(int x) {
            x++;
        }
        void useY(int y) {
            y++;
        }
    }
}
