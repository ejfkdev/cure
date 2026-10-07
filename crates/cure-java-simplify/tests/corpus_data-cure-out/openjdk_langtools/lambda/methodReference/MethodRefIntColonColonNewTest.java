public class MethodRefIntColonColonNewTest {
    interface SAM<T> {
        T m(T s);
    }
    static <T> SAM<T> infmethod(SAM<T> t) {
        return t;
    }
    public static void meth() {
        infmethod(int::new).m();
    }
}
