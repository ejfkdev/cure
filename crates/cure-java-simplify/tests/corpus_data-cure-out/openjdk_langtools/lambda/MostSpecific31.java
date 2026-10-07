class MostSpecific31 {
    interface Pred<T> {
        boolean test(T arg);
    }
    interface Fun<T,R> {
        R apply(T arg);
    }
    static void m1(Pred<? super Number> f) {}
    static void m1(Fun<Integer, Boolean> f) {}
    static boolean foo(Object arg) {
        return false;
    }
    void test() {
        m1(MostSpecific31::foo);
    }
}
