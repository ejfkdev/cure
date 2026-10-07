strictfp
public class LambdaTestStrictFP {
    static double fld = eval(() -> {
    return Double.longBitsToDouble(0x1e7ee00000000000L) * Double.longBitsToDouble(0x2180101010101010L);
});
    public static void main(String[] args) {
        double result = eval(() -> {
            return Double.longBitsToDouble(0x1e7ee00000000000L) * Double.longBitsToDouble(0x2180101010101010L);
        });
        {
            double z = Double.longBitsToDouble(0x1e7ee00000000000L) * Double.longBitsToDouble(0x2180101010101010L);
            check(z, result, "method");
            check(z, fld, "field");
        }
    }
    private static void check(double expected, double got, String where) {
        if (got != expected) {
            throw new AssertionError(where + ": Non-strictfp " + got + " != " + expected);
        }
    }
    private static double eval(Face arg) {
        return arg.m();
    }
    private interface Face {
        double m();
    }
}
