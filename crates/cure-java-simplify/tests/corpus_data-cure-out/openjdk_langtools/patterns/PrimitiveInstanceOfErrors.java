public class PrimitiveInstanceOfErrors {
    public static boolean unboxingAndNarrowingPrimitiveNotAllowedPerCastingConversion() {
        return 42L instanceof int && !(999999999999999999L instanceof int);
    }
    public static <T extends Integer> boolean wideningReferenceConversionUnboxingAndNarrowingPrimitive(T i) {
        return i instanceof byte;
    }
    public static void boxingConversionsBetweenIncompatibleTypes() {
        boolean ret3 = 42 instanceof Short;
    }
}
