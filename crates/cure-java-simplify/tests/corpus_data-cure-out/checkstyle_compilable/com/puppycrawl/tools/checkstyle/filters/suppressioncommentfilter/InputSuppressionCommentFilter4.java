package com.puppycrawl.tools.checkstyle.filters.suppressioncommentfilter;

class InputSuppressionCommentFilter4 {
    private int I;
    private int J;
    private int K;
    private int L;
    private static final int m = 0;
    private int M2;
    private static final int n = 0;
    private int P;
    private int Q;
    private int R;
    private static final int s = 0;
    private int T;
    public static void doit1(int aInt) {}
    public static void doit2(int aInt) {}
    public void doit3() {
        try {
            for (int i = 0; i < 10; i++) {
                while (true) {
                    try {} catch (Exception e) {}
                }
            }
        } catch (Exception ex) {}
        try {} catch (RuntimeException ex) {} catch (Exception ex) {}
    }
    public void doit4() {
        try {} catch (Exception e) {}
    }
}
