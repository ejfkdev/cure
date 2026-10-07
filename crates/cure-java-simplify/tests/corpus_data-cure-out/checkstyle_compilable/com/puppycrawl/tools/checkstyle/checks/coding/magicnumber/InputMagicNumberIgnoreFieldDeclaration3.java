package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

class InputMagicNumberIgnoreFieldDeclaration3 {
    public int hashCode() {
        return 31;
    }
    public int hashCode(int val) {
        return 42;
    }
    public int hashcode() {
        return 13;
    }
    static {
        int x = 21;
    }
    {
        int y = 37;
    }
    public InputMagicNumberIgnoreFieldDeclaration3() {
        int z = 101;
    }
    @InputMagicNumberIntMethodAnnotation(42)
    public void another() {}
    @InputMagicNumberIntMethodAnnotation(value=43)
    public void another2() {}
    @InputMagicNumberIntMethodAnnotation(-44)
    public void anotherNegative() {}
    @InputMagicNumberIntMethodAnnotation(value=-45)
    public void anotherNegative2() {}
}

class TestMethodCallIgnoreFieldDeclaration3 {
    public TestMethodCallIgnoreFieldDeclaration3(int x) {}
    public void method2() {
        TestMethodCallIgnoreFieldDeclaration3 dummyObject = new TestMethodCallIgnoreFieldDeclaration3(62);
    }
}

class BinaryIgnoreFieldDeclaration3 {
    int intValue = 0b101;
    long l = 0b1010000101000101101000010100010110100001010001011010000101000101L;
}

@interface AnnotationWithDefaultValueIgnoreFieldDeclaration3 {
    int value() default 101;
    int[] ar() default {102};
}

class AIgnoreFieldDeclaration3 {
    {
        switch (Blah2IgnoreFieldDeclaration1.LOW) {
            default:
                int b = 122;
        }
    }
}
