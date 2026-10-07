class MethodReference20<X> {
    MethodReference20(X x) {}
    interface SAM<Z> {
        MethodReference20<Z> m(Z z);
    }
    static void test(SAM<Integer> s) {}
    public static void meth() {
        test(MethodReference20::new);
    }
}
