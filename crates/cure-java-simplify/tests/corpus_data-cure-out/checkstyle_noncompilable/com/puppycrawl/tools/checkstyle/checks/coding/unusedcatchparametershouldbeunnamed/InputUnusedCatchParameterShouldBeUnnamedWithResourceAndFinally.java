package com.puppycrawl.tools.checkstyle.checks.coding.unusedcatchparametershouldbeunnamed;

import java.io.File;

public class InputUnusedCatchParameterShouldBeUnnamedWithResourceAndFinally {
    private EOne e;
    Exception exception;
    void testMultiCatch() {
        try {
            File file = new File("file.txt");
        } catch (NullPointerException | IllegalArgumentException e) {
            this.e.printStackTrace();
        }
        try {
            File file = new File("file.txt");
        } catch (NullPointerException | IllegalArgumentException _) {
            this.e.printStackTrace();
        }
        try {
            File file = new File("file.txt");
        } catch (NullPointerException | IllegalArgumentException e) {
            e.printStackTrace();
        }
        try {
            File file = new File("file.txt");
        } catch (NullPointerException e) {
            System.out.println("null pointer exception");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
    void testTryWithResource() {
        try (var a = lock()) {} catch (Exception e) {}
        try (var a = lock()) {} catch (Exception e) {
            System.out.println(e.toString());
        }
        try (var a = lock()) {} catch (Exception _) {
            System.out.println(e.toString());
        }
    }
    void tryCatchFinally() {
        try {
            File file = new File("file.txt");
        } catch (NullPointerException | IllegalArgumentException e) {
            this.e.printStackTrace();
        } finally {
            System.out.println(e);
        }
        try {
            File file = new File("file.txt");
        } catch (NullPointerException | IllegalArgumentException e) {
            e.printStackTrace();
        } finally {
            System.out.println("close the file");
        }
    }
    void e() {}
    AutoCloseable lock() {
        return null;
    }
}

class EOne {
    void printStackTrace() {}
}

class AOne {
}
