public class RecordPatternsExhaustiveSwitch {
    class A {
    }
    class B extends A {
    }
    sealed interface I permits C, D {
    }
    final class C implements I {
    }
    final class D implements I {
    }
    record Pair<T>(T x, T y) {
    }
    static void test() {
        Pair<A> p1 = null;
        Pair<I> p2 = null;
        switch (p1) {
            case Pair<A>(A a, B b) -> System.out.println("a");
            case Pair<A>(B b, A a) -> System.out.println("a");
            case Pair<A>(A a1, A a2) -> System.out.println("exhaustive now");
        }
        switch (p2) {
            case Pair<I>(I i, C c) -> System.out.println("a");
            case Pair<I>(I i, D d) -> System.out.println("a");
        }
        switch (p2) {
            case Pair<I>(C c, I i) -> System.out.println("a");
            case Pair<I>(D d, C c) -> System.out.println("a");
            case Pair<I>(D d1, D d2) -> System.out.println("a");
        }
    }
}
