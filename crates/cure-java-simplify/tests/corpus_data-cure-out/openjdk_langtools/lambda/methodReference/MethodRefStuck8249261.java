class MethodRefStuck8249261 {
    void p(int padding) {}
    static boolean t() {
        return true;
    }
    private void test() {
        p(MethodRefStuck8249261::t);
        p(MethodRefStuck8249261::t);
        p(MethodRefStuck8249261::t + 1);
        p(MethodRefStuck8249261::t ? 1 : 0);
        p(MethodRefStuck8249261::t);
        p(switch (MethodRefStuck8249261::t) {
            default -> 0;
        });
        p(() -> true);
        p(() -> true);
        p((() -> true) + 1);
        p((() -> true) ? 1 : 0);
        p(() -> true);
        p(switch (() -> true) {
            default -> 0;
        });
    }
}
