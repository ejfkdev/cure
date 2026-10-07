public class T8317300 {
    record Foo(int x) {
    }
    record Bar(Foo x) {
    }
    void test1(Object obj) {
        switch (obj) {
            case Foo(int x) -> {}
            default -> {}
        }
    }
    void test2(Object obj) {
        switch (obj) {
            case Bar(final Foo(int x)) -> {}
            default -> {}
        }
    }
}
