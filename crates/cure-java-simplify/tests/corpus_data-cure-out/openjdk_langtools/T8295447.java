public class T8295447 {
    class Foo {
        void m(Object o) {}
        Foo(Object o) {
            m(o instanceof Foo(int x) ? 0 : 1);
        }
        void m(int i) {}
    }
    class Base {
        int i;
        Base(int j) {
            i = j;
        }
    }
    class Sub extends Base {
        Sub(Object o) {
            super(o instanceof java.awt.Point(int x, int y) ? x + y : 0);
        }
    }
}
