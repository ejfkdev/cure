package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

class InputMagicNumberIntegersOnly3 {
    public int hashCode() {
        return 31;
    }
    public int hashCode(int val) {
        return 42;
    }
    public int hashcode() {
        return 13;
    }
    static {}
    {}
    public InputMagicNumberIntegersOnly3() {}
    @InputMagicNumberIntMethodAnnotation(42)
    public void another() {}
    @InputMagicNumberIntMethodAnnotation(value=43)
    public void another2() {}
    @InputMagicNumberIntMethodAnnotation(-44)
    public void anotherNegative() {}
    @InputMagicNumberIntMethodAnnotation(value=-45)
    public void anotherNegative2() {}
}

class TestMethodCallIntegersOnly3 {
    public TestMethodCallIntegersOnly3(int x) {}
    public void method2() {
        TestMethodCallIntegersOnly3 dummyObject = new TestMethodCallIntegersOnly3(62);
    }
}

class BinaryIntegersOnly3 {
    int intValue = 0b101;
    long l = 0b1010000101000101101000010100010110100001010001011010000101000101L;
}

@interface AnnotationWithDefaultValueIntegersOnly3 {
    int value() default 101;
    int[] ar() default {102};
}

class AIntegersOnly3 {
    {
        switch (Blah2IntegersOnly1.LOW) {
            default:
                int b = 122;
        }
    }
}
