public class LambdaTestStrictFPMethod {
    public static void main(String[] args) {
        new LambdaTestStrictFPMethod().test();
    }
    strictfp void test() {
        double result = eval(() -> {
            return Double.longBitsToDouble(0x1e7ee00000000000L) * Double.longBitsToDouble(0x2180101010101010L);
        });
        check(Double.longBitsToDouble(0x1e7ee00000000000L) * Double.longBitsToDouble(0x2180101010101010L), result, "method");
    }
    strictfp void check(double expected, double got, String where) {
        if (got != expected) {
            throw new AssertionError(where + ": Non-strictfp " + got + " != " + expected);
        }
    }
    static double eval(Face arg) {
        return arg.m();
    }
    interface Face {
        double m();
    }
}
