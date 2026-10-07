import java.util.List;

public class T8345474 {
    public static void main(String[] args) {
        erasureInstanceofTypeComparisonOperator();
    }
    public static void erasureInstanceofTypeComparisonOperator() {
        assertTrue(List.of((short) 42).get(0) instanceof int);
    }
    static void assertTrue(boolean actual) {
        if (!actual) {
            throw new AssertionError("Expected: true, but got false");
        }
    }
}
