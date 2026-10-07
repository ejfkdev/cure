public class LambdaConv03 {
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
    static {
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        try {
            exec((Object x) -> {
                return x.hashCode();
            }, null);
        } catch (RuntimeException e) {
            assertTrue(true);
        }
    }
    {
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        try {
            exec((Object x) -> {
                return x.hashCode();
            }, null);
        } catch (RuntimeException e) {
            assertTrue(true);
        }
    }
    public static void test1() {
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        try {
            exec((Object x) -> {
                return x.hashCode();
            }, null);
        } catch (RuntimeException e) {
            assertTrue(true);
        }
    }
    public void test2() {
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        assertTrue(3 == exec((Integer x) -> {
            return x;
        }, 3));
        try {
            exec((Object x) -> {
                return x.hashCode();
            }, null);
        } catch (RuntimeException e) {
            assertTrue(true);
        }
    }
    public static void main(String[] args) {
        test1();
        new LambdaConv03().test2();
        assertTrue(assertionCount == 12);
    }
}
