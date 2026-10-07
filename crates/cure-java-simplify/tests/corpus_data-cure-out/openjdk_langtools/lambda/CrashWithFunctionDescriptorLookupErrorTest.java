import java.util.List;

class CrashWithFunctionDescriptorLookupErrorTest {
    void m() {
        test(List.of(new X()).get(0));
    }
    void test(A<?> a) {}
    void test(B<?> b) {}
    interface A<T extends A<T>> {
        T a();
    }
    interface B<T extends B<T>> {
        T b();
    }
    class X implements A<X>, B<X> {
        public X a() {
            return null;
        }
        public X b() {
            return null;
        }
    }
}
