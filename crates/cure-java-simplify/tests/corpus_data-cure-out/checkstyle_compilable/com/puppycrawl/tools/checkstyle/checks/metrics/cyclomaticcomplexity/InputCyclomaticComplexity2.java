package com.puppycrawl.tools.checkstyle.checks.metrics.cyclomaticcomplexity;

public class InputCyclomaticComplexity2 {
    public InputCyclomaticComplexity2() {
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
    public InputCyclomaticComplexity2(int aParam) {
        new Thread(new Runnable() {
            // NP = 2
            public void run() { // violation 'Cyclomatic Complexity is 2 (max allowed is 0).'
                // NP(while-statement) = (while-range=1) + (expr=0) + 1 = 2
                while (true) {
                }
            }
        }).start();
    }
}
