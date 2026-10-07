import java.lang.reflect.Array;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestInnerVarArgsThis {
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
    class CIA {
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
        public class CIB {
            public void testVarArgsNsSuperclass() {
                NsII q = CIA.this::xvO;
                assertEquals("xvO:55*66*", q.m(55, 66));
            }
            public void testVarArgsNsArray() {
                Nsai q = CIA.this::xvO;
                assertEquals("xvO:[55,66,]*", q.m(new int[] {55, 66}));
            }
            public void testVarArgsNsII() {
                NsII q = CIA.this::xvI;
                assertEquals("xvI:33-7-", q.m(33, 7));
                q = CIA.this::xIvI;
                assertEquals("xIvI:5040-", q.m(50, 40));
                q = CIA.this::xvi;
                assertEquals("xvi:123", q.m(100, 23));
                q = CIA.this::xIvi;
                assertEquals("xIvi:(9)21", q.m(9, 21));
            }
            public void testVarArgsNsiii() {
                Nsiii q = CIA.this::xvI;
                assertEquals("xvI:3-2-1-", q.m(3, 2, 1));
                q = CIA.this::xIvI;
                assertEquals("xIvI:88899-2-", q.m(888, 99, 2));
                q = CIA.this::xvi;
                assertEquals("xvi:987", q.m(900, 80, 7));
                q = CIA.this::xIvi;
                assertEquals("xIvi:(333)99", q.m(333, 27, 72));
            }
            public void testVarArgsNsi() {
                Nsi q = CIA.this::xvI;
                assertEquals("xvI:3-", q.m(3));
                q = CIA.this::xIvI;
                assertEquals("xIvI:888", q.m(888));
                q = CIA.this::xvi;
                assertEquals("xvi:900", q.m(900));
                q = CIA.this::xIvi;
                assertEquals("xIvi:(333)0", q.m(333));
            }
            public void testVarArgsNsaO() {
                NsaO q = CIA.this::xvO;
                assertEquals("xvO:yo*there*dude*", q.m(new String[] {"yo", "there", "dude"}));
            }
        }
        CIB cib() {
            return new CIB();
        }
        class E {
            String xI(Integer i) {
                return "ExI:" + i;
            }
        }
    }
    CIA cia() {
        return new CIA();
    }
    @Test
    public void testVarArgsNsSuperclass() {
        cia().cib().testVarArgsNsSuperclass();
    }
    @Test
    public void testVarArgsNsArray() {
        cia().cib().testVarArgsNsArray();
    }
    @Test
    public void testVarArgsNsII() {
        cia().cib().testVarArgsNsII();
    }
    @Test
    public void testVarArgsNsiii() {
        cia().cib().testVarArgsNsiii();
    }
    @Test
    public void testVarArgsNsi() {
        cia().cib().testVarArgsNsi();
    }
    @Test
    public void testVarArgsNsaO() {
        cia().cib().testVarArgsNsaO();
    }
}
