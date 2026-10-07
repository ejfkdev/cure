class LambdaEffectivelyFinalTest {
    interface SAM {
        int m();
    }
    void foo(LambdaEffectivelyFinalTest.SAM s) {}
    void m1(int x) {
        foo(() -> x + 1);
    }
    void m2(int x) {
        foo(() -> x + 1);
    }
    void m3(int x, boolean cond) {
        int y;
        if (cond) 
            y = 1;
        foo(() -> x + y);
    }
    void m4(int x, boolean cond) {
        foo(() -> x + (cond ? 1 : 2));
    }
    void m5(int x, boolean cond) {
        int y;
        if (cond) 
            y = 1;
        foo(() -> x + 2);
    }
    void m6(int x) {
        foo(() -> x + 1);
        x++;
    }
    void m7(int x) {
        foo(() -> x = 1);
    }
    void m8() {
        int y;
        foo(() -> y = 1);
    }
}
