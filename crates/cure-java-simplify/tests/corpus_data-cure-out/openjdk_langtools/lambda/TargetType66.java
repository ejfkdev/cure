class TargetType66 {
    interface SAM1 {
        void m(String s);
    }
    interface SAM2 {
        void m(Integer s);
    }
    void g(SAM1 s1) {}
    void g(SAM2 s2) {}
    void test() {
        g((x) -> {});
        g((x) -> {});
        g((x) -> {});
        g((x) -> {});
        g((x) -> {});
    }
}
