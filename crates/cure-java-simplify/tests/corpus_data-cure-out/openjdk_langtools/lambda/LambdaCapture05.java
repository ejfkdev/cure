public class LambdaCapture05 {
    static int assertionCount = 0;
    static void assertTrue(boolean cond) {
        assertionCount++;
        if (!cond) 
            throw new AssertionError();
    }
    interface TU<T, U> {
        public T foo(U u);
    }
    public static <T, U> T exec(TU<T, U> lambda, U x) {
        return lambda.foo(x);
    }
    int i = 40;
    void test1(final int a0) {
        exec((final Integer a1) -> {
            exec((final Integer a2) -> {
                exec((final Integer a3) -> {
                    assertTrue(106 == a0 + a1 + a2 + a3 + 10 + 20 + i);
                    return null;
                }, 3);
                return null;
            }, 2);
            return null;
        }, 1);
    }
    static void test2(final int a0) {
        exec((final Integer a1) -> {
            exec((final Integer a2) -> {
                exec((final Integer a3) -> {
                    assertTrue(66 == a0 + a1 + a2 + a3 + 10 + 20);
                    return null;
                }, 3);
                return null;
            }, 2);
            return null;
        }, 1);
    }
    public static void main(String[] args) {
        new LambdaCapture05().test1(30);
        test2(30);
        assertTrue(assertionCount == 2);
    }
}
