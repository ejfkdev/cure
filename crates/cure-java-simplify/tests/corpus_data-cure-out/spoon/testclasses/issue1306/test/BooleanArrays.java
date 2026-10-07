package spoon.test.ctClass.testclasses.issue1306.test;

public final class BooleanArrays {
    private static final boolean[] EMPTY = {};
    public static boolean[] arrayOf(boolean... values) {
        return values;
    }
    public static boolean[] emptyArray() {
        return EMPTY;
    }
    private BooleanArrays() {}
}
