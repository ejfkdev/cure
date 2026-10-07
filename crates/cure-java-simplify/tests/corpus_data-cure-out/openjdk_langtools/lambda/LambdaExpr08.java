class LambdaExpr08 {
    interface SAM {
        String m();
    }
    void test() {
        SAM sam = () -> "";
    }
}
