class MethodReference64 {
    interface ClassFactory {
        Object m();
    }
    interface ArrayFactory {
        Object m(int i);
    }
    @interface Anno {
    }
    enum E {
    }
    interface I {
    }
    static class Foo<X> {
    }
    void m(ClassFactory cf) {}
    void m(ArrayFactory cf) {}
    void testAssign() {
        ArrayFactory a2 = Foo[]::new;
    }
    void testMethod() {
        m(Anno::new);
        m(E::new);
        m(I::new);
        m(Foo::new);
        m(1::new);
        m(Foo[]::new);
        m(Foo[]::new);
    }
}
