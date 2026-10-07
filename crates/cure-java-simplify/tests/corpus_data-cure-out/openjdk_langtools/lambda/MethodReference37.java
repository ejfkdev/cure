class MethodReference37 {
    interface SAM1<R> {
        R invoke();
    }
    interface SAM2<R, A> {
        R invoke(A a);
    }
    static class Outer {
        class Inner {
        }
        void test1() {}
        void test2() {}
    }
    static void test1() {
        SAM2<Outer.Inner, Outer> sam = Outer.Inner::new;
    }
    void test2() {
        SAM2<Outer.Inner, Outer> sam1 = Outer.Inner::new;
    }
}
