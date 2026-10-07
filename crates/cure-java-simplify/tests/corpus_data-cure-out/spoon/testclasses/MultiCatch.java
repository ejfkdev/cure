package spoon.test.imports.testclasses;

import java.security.AccessControlException;

public class MultiCatch {
    public void test() {
        try {} catch (ArithmeticException | AccessControlException e) {}
    }
}
