class Conditional02 {
    <Z> void m1(Z z) {}
    <Z> void m2(Z... z) {}
    void test(boolean flag) {
        m1("");
        m2("");
        m2("", "");
    }
}
