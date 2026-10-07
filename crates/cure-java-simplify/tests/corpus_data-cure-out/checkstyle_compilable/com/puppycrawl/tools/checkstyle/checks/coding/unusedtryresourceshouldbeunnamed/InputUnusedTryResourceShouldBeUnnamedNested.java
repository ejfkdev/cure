package com.puppycrawl.tools.checkstyle.checks.coding.unusedtryresourceshouldbeunnamed;

public class InputUnusedTryResourceShouldBeUnnamedNested {
    void testNested() {
        try (AutoCloseable a = lock()) {
            System.out.println(a);
            try (AutoCloseable b = lock()) {} catch (Exception e) {
                System.out.println(e);
            }
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }
    }
    void testNested2() {
        try {
            try {
                try {
                    try (AutoCloseable a = lock()) {} catch (Exception e) {
                        System.out.println(e);
                    }
                } catch (Exception e) {
                    System.out.println(e.getMessage());
                }
            } catch (Exception e) {
                System.out.println(e);
            }
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }
    }
    void testNested3() {
        try (AutoCloseable a = lock()) {
            System.out.println(a);
            try (AutoCloseable b = lock()) {
                try (AutoCloseable c = lock()) {
                    System.out.println(c);
                    try (AutoCloseable d = lock()) {} catch (Exception e) {
                        System.out.println(e);
                    }
                } catch (Exception e) {
                    System.out.println(e.getMessage());
                }
            } catch (Exception e) {
                System.out.println(e);
            }
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }
    }
    AutoCloseable lock() {
        return null;
    }
}
