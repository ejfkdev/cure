public class T8023549 {
    static class Foo<X> {
    }
    interface Supplier<X> {
        X make();
    }
    interface ExtSupplier<X> extends Supplier<X> {
    }
    void m1(Supplier<Foo<String>> sfs) {}
    void m2(Supplier<Foo<String>> sfs) {}
    void m2(ExtSupplier<Foo<Integer>> sfs) {}
    void test() {
        m1(Foo::new);
        m2(Foo::new);
    }
}
