public class T8036019 {
    enum E {
        E(String value) {  }
    }
    interface A {
    }
    interface B {
    }
    public class Foo<T> {
        Foo<? extends A|B> foo1 = null;
    }
    @SuppressWarnings({,0})
    public class AV {
    }
}
