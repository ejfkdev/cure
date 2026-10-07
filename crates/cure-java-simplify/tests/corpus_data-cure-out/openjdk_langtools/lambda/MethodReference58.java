class MethodReference58 {
    interface F_Object {
        <X> void m(X x);
    }
    interface F_Integer {
        <X extends Integer> void m(X x);
    }
    void test() {}
    <Z extends Number> void g(Z z) {}
}
