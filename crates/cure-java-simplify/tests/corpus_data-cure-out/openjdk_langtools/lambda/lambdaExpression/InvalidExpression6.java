public class InvalidExpression6 {
    interface SAM {
        void m(int i);
    }
    void test() {
        SAM s = (int n) -> {
            continue;
        };
    }
}
