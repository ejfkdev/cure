import java.lang.reflect.Array;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

class MethodReferenceTestVarArgsSuper_Sub {
    String xvI(Integer... vi) {
        StringBuilder sb = new StringBuilder("xvI:");
        for (Integer i : vi) {
            sb.append(i);
            sb.append("-");
        }
        return sb.toString();
    }
    String xIvI(Integer f, Integer... vi) {
        StringBuilder sb = new StringBuilder("xIvI:");
        sb.append(f);
        for (Integer i : vi) {
            sb.append(i);
            sb.append("-");
        }
        return sb.toString();
    }
    String xvi(int... vi) {
        int sum = 0;
        for (int i : vi) {
            sum += i;
        }
        return "xvi:" + sum;
    }
    String xIvi(Integer f, int... vi) {
        int sum = 0;
        for (int i : vi) {
            sum += i;
        }
        return "xIvi:(" + f + ")" + sum;
    }
    String xvO(Object... vi) {
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
}

public class MethodReferenceTestVarArgsSuper extends MethodReferenceTestVarArgsSuper_Sub {
    interface SPRII {
        String m(Integer a, Integer b);
    }
    interface SPRiii {
        String m(int a, int b, int c);
    }
    interface SPRi {
        String m(int a);
    }
    interface SPRaO {
        String m(Object[] a);
    }
    interface SPRai {
        String m(int[] a);
    }
    interface SPRvi {
        String m(int... va);
    }
    String xvI(Integer... vi) {
        return "ERROR";
    }
    String xIvI(Integer f, Integer... vi) {
        return "ERROR";
    }
    String xvi(int... vi) {
        return "ERROR";
    }
    String xIvi(Integer f, int... vi) {
        return "ERROR";
    }
    String xvO(Object... vi) {
        return "ERROR";
    }
    @Test
    public void testVarArgsSPRSuperclass() {
        SPRII q = super::xvO;
        assertEquals("xvO:55*66*", q.m(55, 66));
    }
    @Test
    public void testVarArgsSPRArray() {
        SPRai q = super::xvO;
        assertEquals("xvO:[55,66,]*", q.m(new int[] {55, 66}));
    }
    @Test
    public void testVarArgsSPRII() {
        SPRII q = super::xvI;
        assertEquals("xvI:33-7-", q.m(33, 7));
        q = super::xIvI;
        assertEquals("xIvI:5040-", q.m(50, 40));
        q = super::xvi;
        assertEquals("xvi:123", q.m(100, 23));
        q = super::xIvi;
        assertEquals("xIvi:(9)21", q.m(9, 21));
    }
    @Test
    public void testVarArgsSPRiii() {
        SPRiii q = super::xvI;
        assertEquals("xvI:3-2-1-", q.m(3, 2, 1));
        q = super::xIvI;
        assertEquals("xIvI:88899-2-", q.m(888, 99, 2));
        q = super::xvi;
        assertEquals("xvi:987", q.m(900, 80, 7));
        q = super::xIvi;
        assertEquals("xIvi:(333)99", q.m(333, 27, 72));
    }
    @Test
    public void testVarArgsSPRi() {
        SPRi q = super::xvI;
        assertEquals("xvI:3-", q.m(3));
        q = super::xIvI;
        assertEquals("xIvI:888", q.m(888));
        q = super::xvi;
        assertEquals("xvi:900", q.m(900));
        q = super::xIvi;
        assertEquals("xIvi:(333)0", q.m(333));
    }
    @Test
    public void testVarArgsSPRaO() {
        SPRaO q = super::xvO;
        assertEquals("xvO:yo*there*dude*", q.m(new String[] {"yo", "there", "dude"}));
    }
}
