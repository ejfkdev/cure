package com.puppycrawl.tools.checkstyle.checks.coding.unusedcatchparametershouldbeunnamed;

public class InputUnusedCatchParameterShouldBeUnnamed {
    private ETwo e;
    Exception exception;
    void test(Object o) {
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            System.out.println("infinity");
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {}
        try {
            int x = 1 / 0;
        } catch (@SuppressWarnings("") Exception e) {
            this.e.printStackTrace();
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            System.out.println(e);
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            e = new Exception();
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            exception = e;
        }
        try {
            int x = 1 / 0;
        } catch (Exception A) {
            ATwo a;
        }
        try {
            int x = 1 / 0;
        } catch (Exception A) {
            Object a = new ATwo();
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            System.out.println(this.e);
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            try {
                int y = 1 / 0;
            } catch (Exception _) {
                System.out.println(e.getLocalizedMessage());
            }
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            test(e);
        }
        try {
            int x = 1 / 0;
        } catch (Exception e) {
            e();
        }
        try {
            int x = 1 / 0;
        } catch (Exception _) {
            System.out.println("infinity");
        }
    }
    void e() {}
}

class ETwo {
    void printStackTrace() {}
}

class ATwo {
}
