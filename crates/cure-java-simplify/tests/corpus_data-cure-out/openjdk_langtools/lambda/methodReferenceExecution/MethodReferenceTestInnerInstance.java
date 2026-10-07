import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestInnerInstance {
    @Test
    public void testMethodReferenceInnerInstance() {
        cia().cib().testMethodReferenceInstance();
    }
    @Test
    public void testMethodReferenceInnerExternal() {
        cia().cib().testMethodReferenceExternal();
    }
    interface SI {
        String m(Integer a);
    }
    class CIA {
        String xI(Integer i) {
            return "xI:" + i;
        }
        public class CIB {
            public void testMethodReferenceInstance() {
                SI q = CIA.this::xI;
                assertEquals("xI:55", q.m(55));
            }
            public void testMethodReferenceExternal() {
                SI q = new E()::xI;
                assertEquals("ExI:77", q.m(77));
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
}
