class TargetType33 {
    interface A<X> {
        X m();
    }
    void m(A<Integer> a) {}
    <Z> void m2(A<Z> a) {}
    int intRes(Object o) {
        return 42;
    }
    void testMethodRef(boolean flag) {
        m(this::intRes);
        m2(this::intRes);
    }
}
