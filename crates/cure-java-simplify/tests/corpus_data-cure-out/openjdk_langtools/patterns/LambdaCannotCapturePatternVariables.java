import java.util.function.Supplier;

public class LambdaCannotCapturePatternVariables {
    static Number num1 = 1;
    static Number num2 = null;
    static Number staticNum1 = num1 instanceof Integer i ? ((Supplier<Integer>) (() -> i)).get() : null;
    static Number staticNum2 = num2 instanceof Integer i ? ((Supplier<Integer>) (() -> i)).get() : null;
    Number instanceNum1 = num1 instanceof Integer i ? ((Supplier<Integer>) (() -> i)).get() : null;
    Number instanceNum2 = num2 instanceof Integer i ? ((Supplier<Integer>) (() -> i)).get() : null;
    public static void main(String[] args) {
        var testVar = new LambdaCannotCapturePatternVariables();
        testVar.testInstanceOfPatternVariable(Integer.valueOf(1));
        testVar.testSwitchPatternVariable(Integer.valueOf(1));
        testVar.test(Integer.valueOf(1));
        assertTrue(staticNum1 != null, "staticNum1 is null unexpectedly");
        assertTrue(staticNum2 == null, "staticNum1 is not null unexpectedly");
        assertTrue(testVar.instanceNum1 != null, "instanceNum1 is null unexpectedly");
        assertTrue(testVar.instanceNum2 == null, "instanceNum2 is not null unexpectedly");
        assertTrue(staticNum1.intValue() == 1, "staticNum1.intValue() is not equal to 1");
        assertTrue(testVar.instanceNum1.intValue() == 1, "instanceNum1.intValue() is not equal to 1");
    }
    public Integer testInstanceOfPatternVariable(Object x) {
        return x instanceof Number y ? ((Supplier<Integer>) (() -> {
            return y instanceof Integer z ? z : 1;
        })).get() : null;
    }
    public Integer testSwitchPatternVariable(Object x) {
        switch (x) {
            case Number n:
                {
                    return ((Supplier<Integer>) (() -> {
                        return n instanceof Integer i ? i : 1;
                    })).get();
                }
            default:
                return null;
        }
    }
    public Integer test(Object x) {
        Integer bar = 1;
        return x instanceof Number y ? ((Supplier<Integer>) (() -> {
            return y instanceof Integer z ? z : bar;
        })).get() : bar;
    }
    static void assertTrue(boolean cond, String info) {
        if (!cond) 
            throw new AssertionError(info);
    }
}
