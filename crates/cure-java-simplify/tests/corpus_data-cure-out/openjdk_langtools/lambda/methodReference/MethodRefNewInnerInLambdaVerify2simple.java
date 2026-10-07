import java.util.function.Function;

public class MethodRefNewInnerInLambdaVerify2simple {
    public static void main(String[] args) {
        new MethodRefNewInnerInLambdaVerify2simple().runTest();
    }
    private void runTest() {}
    private class SomeClass {
        SomeClass() {}
    }
}

interface Sup {
    Object get();
}
