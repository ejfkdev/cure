import java.lang.reflect.Array;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;
import org.junit.jupiter.api.Test;

@SuppressWarnings({"rawtypes", "unchecked"})
public class MethodReferenceTestFDCCE {
    static void assertCCE(Throwable t) {
        assertEquals("java.lang.ClassCastException", t.getClass().getName());
    }
    interface Pred<T> {
        boolean accept(T x);
    }
    interface Ps {
        boolean accept(short x);
    }
    interface Oo {
        Object too(int x);
    }
    interface Reto<T> {
        T m();
    }
    class A {
    }
    class B extends A {
    }
    static boolean isMinor(int x) {
        return x < 18;
    }
    static boolean tst(A x) {
        return true;
    }
    static Object otst(Object x) {
        return x;
    }
    static boolean stst(Short x) {
        return x < 18;
    }
    static short ritst() {
        return 123;
    }
    @Test
    public void testMethodReferenceFDPrim1() {
        Pred<Byte> p = MethodReferenceTestFDCCE::isMinor;
        assertTrue(p.accept((Byte) (byte) 15));
    }
    @Test
    public void testMethodReferenceFDPrim2() {
        Pred<Byte> p = MethodReferenceTestFDCCE::isMinor;
        assertTrue(p.accept((byte) 15));
    }
    @Test
    public void testMethodReferenceFDPrimICCE() {
        Pred<Byte> p = MethodReferenceTestFDCCE::isMinor;
        try {
            p.accept(15);
            fail("Exception should have been thrown");
        } catch (Throwable t) {
            assertCCE(t);
        }
    }
    @Test
    public void testMethodReferenceFDPrimOCCE() {
        Pred<Byte> p = MethodReferenceTestFDCCE::isMinor;
        try {
            p.accept(new Object());
            fail("Exception should have been thrown");
        } catch (Throwable t) {
            assertCCE(t);
        }
    }
    @Test
    public void testMethodReferenceFDRef() {
        Pred<B> p = MethodReferenceTestFDCCE::tst;
        assertTrue(p.accept(new B()));
    }
    @Test
    public void testMethodReferenceFDRefCCE() {
        Pred<B> p = MethodReferenceTestFDCCE::tst;
        try {
            p.accept(new A());
            fail("Exception should have been thrown");
        } catch (Throwable t) {
            assertCCE(t);
        }
    }
    @Test
    public void testMethodReferenceFDPrimPrim() {
        Ps p = MethodReferenceTestFDCCE::isMinor;
        assertTrue(p.accept((byte) 15));
    }
    @Test
    public void testMethodReferenceFDPrimBoxed() {
        Ps p = MethodReferenceTestFDCCE::stst;
        assertTrue(p.accept((byte) 15));
    }
    @Test
    public void testMethodReferenceFDPrimRef() {
        Oo p = MethodReferenceTestFDCCE::otst;
        assertEquals("java.lang.Integer", p.too(15).getClass().getName());
    }
    @Test
    public void testMethodReferenceFDRet1() {
        Reto<Short> p = MethodReferenceTestFDCCE::ritst;
        assertEquals((Short) (short) 123, p.m());
    }
}
