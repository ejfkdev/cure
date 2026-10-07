class BadSwitchExpressionLambda {
    interface SAM {
        void invoke();
    }
    public static void m() {}
    public static void r(SAM sam) {}
    void test(int i) {
        SAM sam1 = () -> m();
        SAM sam2 = () -> switch (i) {
            case 0 -> m();
            default -> m();
        };
        r(() -> m());
        r(() -> switch (i) {
            case 0 -> m();
            default -> m();
        });
        return switch (i) {
            case 0 -> m();
            default -> m();
        };
    }
}
