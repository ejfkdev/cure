import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

interface DSPRI {
    String m(String a);
}

interface DSPRA {
    default String xsA__(String s) {
        return "A__xsA:" + s;
    }
    default String xsAB_(String s) {
        return "AB_xsA:" + s;
    }
}

interface DSPRB extends DSPRA {
    default String xsAB_(String s) {
        return "AB_xsB:" + s;
    }
    default String xs_B_(String s) {
        return "_B_xsB:" + s;
    }
}

public class MethodReferenceTestSuperDefault implements DSPRB {
    @Test
    public void testMethodReferenceSuper() {
        DSPRI q = DSPRB.super::xsA__;
        assertEquals("A__xsA:*", q.m("*"));
        q = DSPRB.super::xsAB_;
        assertEquals("AB_xsB:*", q.m("*"));
        q = DSPRB.super::xs_B_;
        assertEquals("_B_xsB:*", q.m("*"));
    }
}
