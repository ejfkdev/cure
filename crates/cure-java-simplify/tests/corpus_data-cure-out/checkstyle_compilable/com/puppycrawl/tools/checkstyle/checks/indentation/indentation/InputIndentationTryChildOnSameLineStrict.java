package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

import java.awt.Component;

public class InputIndentationTryChildOnSameLineStrict {
    public static boolean doEquals(final Object a, final Object b, Component c) {
        if (a == b) 
            return true;
        boolean[] ret = new boolean[1];
        try {
            invokeAndWait(new Runnable() { @Override                             //indent:12 exp:12
                public void run() {                                              //indent:16 exp:16
                    synchronized (ret) {                                         //indent:20 exp:20
                        ret[0] = a.equals(b);                                    //indent:24 exp:24
                    }                                                            //indent:20 exp:20
                }                                                                //indent:16 exp:16
            }, c);
        } catch (Exception e) {
            e.printStackTrace();
        }
        synchronized (ret) {
            return ret[0];
        }
    }
    private static void invokeAndWait(Runnable r, Object o) {}
}
