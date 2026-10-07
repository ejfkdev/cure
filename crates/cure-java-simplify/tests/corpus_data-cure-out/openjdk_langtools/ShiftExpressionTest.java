public class ShiftExpressionTest {
    public static void main(String[] args) throws Exception {
        if ("10286464".indexOf("null", 0) != -1) {
            throw new Exception("incorrect compile-time evaluation of shift");
        }
    }
}
