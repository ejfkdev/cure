class MostSpecific19 {
    interface F1 {
        <X extends Number> Object apply(X arg);
    }
    interface F2 {
        <Y extends Integer> String apply(Y arg);
    }
    static void m1(F1 f) {}
    static void m1(F2 f) {}
    static String foo(Object in) {
        return "a";
    }
    void test() {
        m1(MostSpecific19::foo);
    }
}
