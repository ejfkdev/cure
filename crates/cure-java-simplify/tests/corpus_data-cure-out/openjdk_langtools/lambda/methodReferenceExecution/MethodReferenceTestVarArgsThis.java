import java.lang.reflect.Array;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

interface NsII {
    String m(Integer a, Integer b);
}

interface Nsiii {
    String m(int a, int b, int c);
}

interface Nsi {
    String m(int a);
}

interface NsaO {
    String m(Object[] a);
}

interface Nsai {
    String m(int[] a);
}

interface Nsvi {
    String m(int... va);
}

public class MethodReferenceTestVarArgsThis {
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
    @Test
    public void testVarArgsNsSuperclass() {
        NsII q = this::xvO;
        assertEquals("xvO:55*66*", q.m(55, 66));
    }
    @Test
    public void testVarArgsNsArray() {
        Nsai q = this::xvO;
        assertEquals("xvO:[55,66,]*", q.m(new int[] {55, 66}));
    }
    @Test
    public void testVarArgsNsII() {
        NsII q = this::xvI;
        assertEquals("xvI:33-7-", q.m(33, 7));
        q = this::xIvI;
        assertEquals("xIvI:5040-", q.m(50, 40));
        q = this::xvi;
        assertEquals("xvi:123", q.m(100, 23));
        q = this::xIvi;
        assertEquals("xIvi:(9)21", q.m(9, 21));
    }
    @Test
    public void testVarArgsNsiii() {
        Nsiii q = this::xvI;
        assertEquals("xvI:3-2-1-", q.m(3, 2, 1));
        q = this::xIvI;
        assertEquals("xIvI:88899-2-", q.m(888, 99, 2));
        q = this::xvi;
        assertEquals("xvi:987", q.m(900, 80, 7));
        q = this::xIvi;
        assertEquals("xIvi:(333)99", q.m(333, 27, 72));
    }
    @Test
    public void testVarArgsNsi() {
        Nsi q = this::xvI;
        assertEquals("xvI:3-", q.m(3));
        q = this::xIvI;
        assertEquals("xIvI:888", q.m(888));
        q = this::xvi;
        assertEquals("xvi:900", q.m(900));
        q = this::xIvi;
        assertEquals("xIvi:(333)0", q.m(333));
    }
    @Test
    public void testVarArgsNsaO() {
        NsaO q = this::xvO;
        assertEquals("xvO:yo*there*dude*", q.m(new String[] {"yo", "there", "dude"}));
    }
}
