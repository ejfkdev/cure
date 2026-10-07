import java.lang.reflect.Array;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestVarArgs {
    interface SII {
        String m(Integer a, Integer b);
    }
    interface Siii {
        String m(int a, int b, int c);
    }
    interface Si {
        String m(int a);
    }
    interface SaO {
        String m(Object[] a);
    }
    interface Sai {
        String m(int[] a);
    }
    interface Svi {
        String m(int... va);
    }
    static String xvI(Integer... vi) {
        StringBuilder sb = new StringBuilder("xvI:");
        for (Integer i : vi) {
            sb.append(i);
            sb.append("-");
        }
        return sb.toString();
    }
    static String xIvI(Integer f, Integer... vi) {
        StringBuilder sb = new StringBuilder("xIvI:");
        sb.append(f);
        for (Integer i : vi) {
            sb.append(i);
            sb.append("-");
        }
        return sb.toString();
    }
    static String xvi(int... vi) {
        int sum = 0;
        for (int i : vi) {
            sum += i;
        }
        return "xvi:" + sum;
    }
    static String xIvi(Integer f, int... vi) {
        int sum = 0;
        for (int i : vi) {
            sum += i;
        }
        return "xIvi:(" + f + ")" + sum;
    }
    static String xvO(Object... vi) {
        StringBuilder sb = new StringBuilder("xvO:");
        for (Object i : vi) {
            if (i.getClass().isArray()) {
                sb.append("[");
                int len = Array.getLength(i);
                for (int x = 0; x < len; ++x) {
                    sb.append(Array.get(i, x));
                    sb.append(",");
                }
                sb.append("]");
            } else {
                sb.append(i);
            }
            sb.append("*");
        }
        return sb.toString();
    }
    @Test
    public void testVarArgsSuperclass() {
        SII q = MethodReferenceTestVarArgs::xvO;
        assertEquals("xvO:55*66*", q.m(55, 66));
    }
    @Test
    public void testVarArgsArray() {
        Sai q = MethodReferenceTestVarArgs::xvO;
        assertEquals("xvO:[55,66,]*", q.m(new int[] {55, 66}));
    }
    @Test
    public void testVarArgsII() {
        SII q = MethodReferenceTestVarArgs::xvI;
        assertEquals("xvI:33-7-", q.m(33, 7));
        q = MethodReferenceTestVarArgs::xIvI;
        assertEquals("xIvI:5040-", q.m(50, 40));
        q = MethodReferenceTestVarArgs::xvi;
        assertEquals("xvi:123", q.m(100, 23));
        q = MethodReferenceTestVarArgs::xIvi;
        assertEquals("xIvi:(9)21", q.m(9, 21));
    }
    @Test
    public void testVarArgsiii() {
        Siii q = MethodReferenceTestVarArgs::xvI;
        assertEquals("xvI:3-2-1-", q.m(3, 2, 1));
        q = MethodReferenceTestVarArgs::xIvI;
        assertEquals("xIvI:88899-2-", q.m(888, 99, 2));
        q = MethodReferenceTestVarArgs::xvi;
        assertEquals("xvi:987", q.m(900, 80, 7));
        q = MethodReferenceTestVarArgs::xIvi;
        assertEquals("xIvi:(333)99", q.m(333, 27, 72));
    }
    @Test
    public void testVarArgsi() {
        Si q = MethodReferenceTestVarArgs::xvI;
        assertEquals("xvI:3-", q.m(3));
        q = MethodReferenceTestVarArgs::xIvI;
        assertEquals("xIvI:888", q.m(888));
        q = MethodReferenceTestVarArgs::xvi;
        assertEquals("xvi:900", q.m(900));
        q = MethodReferenceTestVarArgs::xIvi;
        assertEquals("xIvi:(333)0", q.m(333));
    }
    @Test
    public void testVarArgsaO() {
        SaO q = MethodReferenceTestVarArgs::xvO;
        assertEquals("xvO:yo*there*dude*", q.m(new String[] {"yo", "there", "dude"}));
    }
}
