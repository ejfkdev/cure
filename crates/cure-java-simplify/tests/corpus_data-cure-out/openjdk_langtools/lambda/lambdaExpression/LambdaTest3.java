public class LambdaTest3 {
    private static int count = 0;
    private static void assertTrue(boolean b) {
        if (!b) 
            throw new AssertionError();
    }
    public static void main(String[] args) {
        Runnable r = (Runnable) (() -> {
            count += 100;
            count += 2;
        });
        assertTrue(count == 0);
        r.run();
        assertTrue(count == 102);
    }
}
