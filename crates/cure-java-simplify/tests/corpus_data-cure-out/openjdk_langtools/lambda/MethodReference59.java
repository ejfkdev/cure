public class MethodReference59 {
    static int assertionCount = 0;
    static void assertTrue(boolean cond) {
        assertionCount++;
        if (!cond) 
            throw new AssertionError();
    }
    interface ArrayFactory<X> {
        X make(int size);
    }
    public static void main(String[] args) {
        ArrayFactory<int[]> factory1 = int[]::new;
        assertTrue(factory1.make(5).length == 5);
        ArrayFactory<int[][]> factory2 = int[][]::new;
        assertTrue(factory2.make(5).length == 5);
        assertTrue(assertionCount == 2);
    }
}
