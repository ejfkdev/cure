package com.puppycrawl.tools.checkstyle.checks.coding.unusedtryresourceshouldbeunnamed;

public class InputUnusedTryResourceShouldBeUnnamed {
    void test() {
        try (AutoCloseable a = lock()) {} catch (Exception e) {}
        try (AutoCloseable b = lock()) {} catch (Exception e) {}
        try (AutoCloseable d = lock()) {
            d.close();
        } catch (Exception e) {
            System.out.println(e.getMessage());
        }
        try (AutoCloseable e = lock()) {} catch (Exception e) {
            System.out.println(e.getMessage());
        }
        try (AutoCloseable _ = lock()) {} catch (Exception e) {}
        try {} catch (Exception e) {}
        try (AutoCloseable a = lock(); AutoCloseable b = lock()) {
            System.out.println(a);
        } catch (Exception e) {}
        try (AutoCloseable a = lock(); AutoCloseable _ = lock()) {
            System.out.println(a);
        } catch (Exception e) {}
    }
    void test2() {
        try (AutoCloseable autoCloseable = lock()) {
            try {} catch (Exception e) {}
            try (AutoCloseable autoCloseable2 = lock()) {} catch (Exception e) {}
            Runnable runnable = new Runnable() {
                @Override
                public void run() {
                    System.out.println(autoCloseable);
                }
            };
        } catch (Exception e) {}
    }
    AutoCloseable lock() {
        return null;
    }
}
