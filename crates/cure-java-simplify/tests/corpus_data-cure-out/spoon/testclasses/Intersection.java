package spoon.test.lambda.testclasses;

import java.util.Collections;
import java.util.List;

public class Intersection<T> {
    public void m() {
        multiInheritedGenericsList().stream().filter((elt) -> elt.test());
    }
    public void m2() {
        ((C & D) (() -> System.out.println())).test();
    }
    public void m3() {
        ((D & C) (() -> System.out.println())).test();
    }
    public void m4() {
        ((E & D) (() -> System.out.println())).test();
    }
    public static <T extends A & B> List<T> multiInheritedGenericsList() {
        return Collections.emptyList();
    }
    public interface A {
        boolean test() throws Exception;
    }
    public interface B {
        boolean test();
    }
    public interface C {
    }
    public interface D {
        void test();
    }
    public interface E {
        void test();
    }
}
