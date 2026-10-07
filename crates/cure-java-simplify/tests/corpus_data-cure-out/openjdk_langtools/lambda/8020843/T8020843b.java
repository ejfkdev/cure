class T8020843b {
    interface Function<X, Y> {
        Y m(X x);
    }
    interface BiFunction<X, Y, Z> {
        Z m(X x, Y y);
    }
    Object m(int i) {
        return null;
    }
    static Object m(String t) {
        return null;
    }
    Object m2(int i) {
        return null;
    }
    static Object m2(long t) {
        return null;
    }
    static void test() {}
}
