class LambdaScope05 {
    interface VoidFun1 {
        void m(int i);
    }
    static Runnable r1 = () -> {};
    Runnable r2 = () -> {};
    static {}
    {}
    static void m_static() {}
    void m() {}
}
