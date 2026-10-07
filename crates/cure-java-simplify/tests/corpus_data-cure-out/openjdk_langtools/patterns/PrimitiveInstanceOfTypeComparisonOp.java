public class PrimitiveInstanceOfTypeComparisonOp {
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
        return 42 instanceof int;
    }
    public static boolean wideningPrimitiveConversion() {
        return (byte) 42 instanceof int && (short) 42 instanceof int && 'a' instanceof int;
    }
    public static boolean narrowingPrimitiveConversion() {
        return 42L instanceof int && !(999999999999999999L instanceof int);
    }
    public static boolean wideningAndNarrowingPrimitiveConversion() {
        return (byte) 42 instanceof char && '*' instanceof byte && !(byte - 42 instanceof char);
    }
    public static boolean boxingConversion() {
        return 42 instanceof Integer;
    }
    public static boolean boxingAndWideningReferenceConversion() {
        int i = 42;
        return i instanceof Object && i instanceof Number && i instanceof Comparable;
    }
    public static boolean unboxing() {
        return Integer.valueOf(1) instanceof int;
    }
    public static boolean unboxingWithObject() {
        Object o1 = 42;
        Object o2 = (byte) 42;
        return o1 instanceof int && o2 instanceof byte && !(o1 instanceof byte && !(o2 instanceof int));
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxing(T i) {
        return i instanceof int;
    }
    public static <T extends Byte> boolean wideningReferenceConversionUnboxing2(T i) {
        return i instanceof byte;
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxing3(T i) {
        return i instanceof float;
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxingAndWideningPrimitive(T i) {
        return i instanceof double;
    }
    public static boolean unboxingAndWideningPrimitiveExact() {
        Byte b = Byte.valueOf((byte) 42);
        Short s = Short.valueOf((short) 42);
        Character c = Character.valueOf('a');
        return b instanceof int && s instanceof int && c instanceof int;
    }
    public static boolean unboxingAndWideningPrimitiveNotExact() {
        return Integer.valueOf(16777217) instanceof float;
    }
    public static boolean unboxingWhenNullAndWideningPrimitive() {
        return !(null instanceof int) && !(null instanceof int) && !(null instanceof int);
    }
    public static boolean narrowingAndUnboxing() {
        return Byte.valueOf((byte) 42) instanceof byte;
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
        return meth() instanceof int;
    }
    static int sideEffect;
    public static Integer methSideEffect() {
        sideEffect++;
        return 42;
    }
    public static boolean exprMethodSideEffect() {
        sideEffect = 5;
        return methSideEffect() instanceof int && sideEffect == 6;
    }
    public class A1 {
        public static int i = 42;
    }
    public static boolean exprStaticallyQualified() {
        return A1.i instanceof int;
    }
    static void assertEquals(boolean expected, boolean actual) {
        if (expected != actual) {
            throw new AssertionError("Expected: " + expected + ", actual: " + actual);
        }
    }
}
