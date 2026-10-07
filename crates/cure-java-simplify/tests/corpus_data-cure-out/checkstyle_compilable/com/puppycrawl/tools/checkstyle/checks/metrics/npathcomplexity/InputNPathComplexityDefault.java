package com.puppycrawl.tools.checkstyle.checks.metrics.npathcomplexity;

public class InputNPathComplexityDefault {
    public void foo() {
        while (true) {
            new Thread(new Runnable() {
               // violation below 'NPath Complexity is 2 (max allowed is 0).'
                public void run() {
                    // NP(while-statement) = (while-range=1) + (expr=0) + 1 = 2
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
    public InputNPathComplexityDefault() {
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
    public InputNPathComplexityDefault(int aParam) {
        new Thread(new Runnable() {
            // violation below 'NPath Complexity is 2 (max allowed is 0).'
            public void run() {
                // NP(while-statement) = (while-range=1) + (expr=0) + 1 = 2
                while (true) {
                }
            }
        }).start();
    }
    public void InputNestedTernaryCheck() {
        double x = getSomething() || Math.random() == 5 ? null : (int) Math.cos(20000);
        double y = 0.2 == Math.random() ? 0.3 == Math.random() ? null : (int) Math.cos(20000) : 6;
        double z = (Integer) (0.2 == Math.random() ? (Integer) null + 0 : 0.3 == Math.random() ? (Integer) null : (int) Math.sin(12600));
    }
    public boolean getSomething() {
        return true;
    }
    public int apply(Object o) {
        return 0;
    }
    public void inClass(int type, Short s, int color) {
        switch (type) {
            case 3:
                new Object() {
                public void anonymousMethod() {
                    // violation above 'NPath Complexity is 3 (max allowed is 0).'
                    {
                        switch (s) {
                        case 5:
                            switch (type) {
                            default:
                            }
                        }
                    }
                }
            };
            default:
                new Object() {
                class SwitchClass {
                    // violation below 'NPath Complexity is 3 (max allowed is 0).'
                    {
                        switch (color) {
                        case 5:
                            switch (type) {
                            default:
                            }
                        }
                    }
                }
            };
        }
    }
}
