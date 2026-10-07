package com.puppycrawl.tools.checkstyle.filters.suppresswithnearbycommentfilter;

public class InputSuppressWithNearbyCommentFilterUsingTagMessageRegexp {
    private int A1;
    private int A2;
    private int A3;
    private int B1;
    private int B2;
    private int B3;
    private int C1;
    private int C2;
    private int C3;
    private int D1;
    private int D2;
    private int D3;
    private static final int e1 = 0;
    private int E2;
    private int E3;
    private static final int e4 = 0;
    private int E5;
    private static final int e6 = 0;
    private int E7;
    private int E8;
    private static final int e9 = 0;
    public static void doit1(int aInt) {}
    public static void doit2(int aInt) {}
    public static void doit3(int aInt) {}
    public void doit4() {
        try {
            for (int i = 0; i < 10; i++) {
                while (true) {
                    try {} catch (Exception e) {} catch (Throwable t) {}
                }
            }
        } catch (Exception ex) {}
    }
}

class Magic9 {
    private int A2;
    private int A1;
}
