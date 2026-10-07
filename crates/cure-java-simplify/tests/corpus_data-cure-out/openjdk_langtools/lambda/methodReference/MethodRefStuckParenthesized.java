public abstract class MethodRefStuckParenthesized {
    interface I {
        String v();
    }
    interface J {
        String v();
    }
    abstract String v();
    abstract void f(I v);
    abstract <X extends J> J g(X x);
    void test() {
        f(g(this::v));
    }
}
