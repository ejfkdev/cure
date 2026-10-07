import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestNew {
    interface M0<T> {
        T m();
    }
    static class N0 {
        N0() {}
    }
    interface M1<T> {
        T m(Integer a);
    }
    static class N1 {
        int i;
        N1(int i) {
            this.i = i;
        }
    }
    interface M2<T> {
        T m(Integer n, String o);
    }
    static class N2 {
        Number n;
        Object o;
        N2(Number n, Object o) {
            this.n = n;
            this.o = o;
        }
        public String toString() {
            return "N2(" + n + "," + o + ")";
        }
    }
    interface MV {
        NV m(Integer ai, int i);
    }
    static class NV {
        int i;
        NV(int... v) {
            i = 0;
            for (int x : v) {
                i += x;
            }
        }
        public String toString() {
            return "NV(" + i + ")";
        }
    }
    @Test
    public void testConstructorReference0() {
        M0<N0> q = N0::new;
        assertEquals("N0", q.m().getClass().getSimpleName());
    }
    @Test
    public void testConstructorReference1() {
        M1<N1> q = N1::new;
        assertEquals("N1", q.m(14).getClass().getSimpleName());
    }
    @Test
    public void testConstructorReference2() {
        M2<N2> q = N2::new;
        assertEquals("N2(7,hi)", q.m(7, "hi").toString());
    }
    @Test
    public void testConstructorReferenceVarArgs() {
        MV q = NV::new;
        assertEquals("NV(50)", q.m(5, 45).toString());
    }
}
