package spoon.test.ctClass.testclasses.issue1306.internal;

public class BooleanArrays {
    private static final BooleanArrays INSTANCE = new BooleanArrays();
    public static BooleanArrays instance() {
        return INSTANCE;
    }
    BooleanArrays() {}
}
