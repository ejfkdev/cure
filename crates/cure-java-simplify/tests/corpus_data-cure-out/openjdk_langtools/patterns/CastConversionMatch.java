public class CastConversionMatch {
    public static void meth() {
        if (42 instanceof int s) {
            System.out.println("Okay");
        } else {
            throw new AssertionError("broken");
        }
        System.out.println(">Test complete");
    }
}
