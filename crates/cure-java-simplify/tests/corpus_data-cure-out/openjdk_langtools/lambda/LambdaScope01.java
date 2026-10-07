public class LambdaScope01 {
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
    public int n = 5;
    public int hashCode() {
        throw new RuntimeException();
    }
    public void test1() {
        try {
            int res = LambdaScope01.exec((Integer x) -> x * hashCode(), 3);
        } catch (RuntimeException e) {
            assertTrue(true);
        }
    }
    public void test2() {
        assertTrue(13 == LambdaScope01.exec((Integer x) -> x + 10, 3));
    }
    public static void main(String[] args) {
        LambdaScope01 t = new LambdaScope01();
        t.test1();
        t.test2();
        assertTrue(assertionCount == 2);
    }
}
