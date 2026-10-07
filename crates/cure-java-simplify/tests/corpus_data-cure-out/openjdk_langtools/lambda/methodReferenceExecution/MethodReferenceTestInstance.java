import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

class MethodReferenceTestInstance_E {
    String xI(Integer i) {
        return "ExI:" + i;
    }
}

public class MethodReferenceTestInstance {
    interface SI {
        String m(Integer a);
    }
    String xI(Integer i) {
        return "xI:" + i;
    }
    @Test
    public void testMethodReferenceInstance() {
        SI q = this::xI;
        assertEquals("xI:55", q.m(55));
    }
    @Test
    public void testMethodReferenceExternal() {
        SI q = new MethodReferenceTestInstance_E()::xI;
        assertEquals("ExI:77", q.m(77));
    }
}
