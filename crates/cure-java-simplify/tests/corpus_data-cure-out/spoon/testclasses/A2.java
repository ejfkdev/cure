package spoon.test.visibility.testclasses;

public class A2 {
    public class B {
        public static final int i = 0;
        public boolean m(Object o) {
            return i == 0;
        }
    }
    public class C<T> {
    }
    public boolean instanceOf(Object o) {
        return o instanceof A2.B;
    }
    public C<String> returnType() {
        return new C<String>();
    }
    public Foo<String>.Bar<String> returnType2(String s) {
        return null;
    }
    public void aMethod() {
        class D {
        		}
        		new D();
    }
}
