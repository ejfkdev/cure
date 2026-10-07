class MethodReference28 {
    interface SAM1 {
        void m(int i);
    }
    interface SAM2 {
        void m(MethodReference28 rec, int i);
    }
    static void static_m1(Integer i) {}
    static void static_m2(Integer i1, Integer i2) {}
    static void static_m3(String s) {}
    static void static_m4(String... ss) {}
    void m1(Integer i) {}
    void m2(Integer i1, Integer i2) {}
    void m3(String s) {}
    void m4(String... ss) {}
    static void testStatic() {
        SAM1 s4 = MethodReference28::static_m4;
    }
    void testBadMember() {
        SAM1 s4 = MethodReference28::m4;
    }
    void testMember() {
        SAM1 s4 = this::m4;
    }
    static void testUnbound() {
        SAM2 s4 = MethodReference28::m4;
    }
}
