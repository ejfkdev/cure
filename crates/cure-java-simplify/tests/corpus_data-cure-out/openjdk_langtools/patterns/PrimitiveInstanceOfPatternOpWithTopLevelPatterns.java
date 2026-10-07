public class PrimitiveInstanceOfPatternOpWithTopLevelPatterns {
    public static final int qualI = 42;
    public static void main(String[] args) {
        assertEquals(true, qualifiedExprConversion());
        assertEquals(true, identityPrimitiveConversion());
        assertEquals(true, wideningPrimitiveConversion());
        assertEquals(true, narrowingPrimitiveConversion());
        assertEquals(true, wideningAndNarrowingPrimitiveConversion());
        assertEquals(true, boxingConversion());
        assertEquals(true, boxingAndWideningReferenceConversion());
        assertEquals(true, unboxing());
        assertEquals(true, unboxingWithObject());
        assertEquals(true, wideningReferenceConversionUnboxing(42));
        assertEquals(true, wideningReferenceConversionUnboxing2(Byte.valueOf((byte) 42)));
        assertEquals(true, wideningReferenceConversionUnboxing3(0x1000000));
        assertEquals(true, wideningReferenceConversionUnboxingAndWideningPrimitive(42));
        assertEquals(true, unboxingAndWideningPrimitiveExact());
        assertEquals(false, unboxingAndWideningPrimitiveNotExact());
        assertEquals(true, unboxingWhenNullAndWideningPrimitive());
        assertEquals(true, narrowingAndUnboxing());
        assertEquals(true, patternExtractRecordComponent());
        assertEquals(true, exprMethod());
        assertEquals(true, exprMethodSideEffect());
        assertEquals(true, exprStaticallyQualified());
    }
    public static boolean qualifiedExprConversion() {
        return PrimitiveInstanceOfTypeComparisonOp.qualI instanceof int;
    }
    public static boolean identityPrimitiveConversion() {
        return 42 instanceof int ii;
    }
    public static boolean wideningPrimitiveConversion() {
        return (byte) 42 instanceof int bb && (short) 42 instanceof int ss && 'a' instanceof int cc;
    }
    public static boolean narrowingPrimitiveConversion() {
        return 42L instanceof int lw && !(999999999999999999L instanceof int lo);
    }
    public static boolean wideningAndNarrowingPrimitiveConversion() {
        return (byte) 42 instanceof char bb && '*' instanceof byte cc && !(byte - 42 instanceof char b2b);
    }
    public static boolean boxingConversion() {
        return 42 instanceof Integer ii;
    }
    public static boolean boxingAndWideningReferenceConversion() {
        int i = 42;
        return i instanceof Object io && i instanceof Number in && i instanceof Comparable cc;
    }
    public static boolean unboxing() {
        return Integer.valueOf(1) instanceof int ii;
    }
    public static boolean unboxingWithObject() {
        Object o1 = 42;
        Object o2 = (byte) 42;
        return o1 instanceof int o1o && o2 instanceof byte o2o && !(o1 instanceof byte o1b && !(o2 instanceof int o2b));
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxing(T i) {
        return i instanceof int ii;
    }
    public static <T extends Byte> boolean wideningReferenceConversionUnboxing2(T i) {
        return i instanceof byte bb;
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxing3(T i) {
        return i instanceof float ff;
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxingAndWideningPrimitive(T i) {
        return i instanceof double ii;
    }
    public static boolean unboxingAndWideningPrimitiveExact() {
        Byte b = Byte.valueOf((byte) 42);
        Short s = Short.valueOf((short) 42);
        Character c = Character.valueOf('a');
        return b instanceof int bb && s instanceof int ss && c instanceof int cc;
    }
    public static boolean unboxingAndWideningPrimitiveNotExact() {
        return Integer.valueOf(16777217) instanceof float ii;
    }
    public static boolean unboxingWhenNullAndWideningPrimitive() {
        return !(null instanceof int bb) && !(null instanceof int ss) && !(null instanceof int cc);
    }
    public static boolean narrowingAndUnboxing() {
        return Byte.valueOf((byte) 42) instanceof byte nn;
    }
    public record P(int i) {
    }
    public static boolean patternExtractRecordComponent() {
        Object p = new P(42);
        return p instanceof P(byte b) && b == 42;
    }
    public static int meth() {
        return 42;
    }
    public static boolean exprMethod() {
        return meth() instanceof int ii;
    }
    static int sideEffect;
    public static Integer methSideEffect() {
        sideEffect++;
        return 42;
    }
    public static boolean exprMethodSideEffect() {
        sideEffect = 5;
        return methSideEffect() instanceof int ii && sideEffect == 6;
    }
    public class A1 {
        public static int i = 42;
    }
    public static boolean exprStaticallyQualified() {
        return A1.i instanceof int ii;
    }
    static void assertEquals(boolean expected, boolean actual) {
        if (expected != actual) {
            throw new AssertionError("Expected: " + expected + ", actual: " + actual);
        }
    }
}
