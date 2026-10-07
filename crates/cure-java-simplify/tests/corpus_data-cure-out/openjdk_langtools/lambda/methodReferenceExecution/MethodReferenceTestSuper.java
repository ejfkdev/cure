import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

interface SPRI {
    String m(String a);
}

class SPRA {
    String xsA__(String s) {
        return "A__xsA:" + s;
    }
    String xsA_M(String s) {
        return "A_MxsA:" + s;
    }
    String xsAB_(String s) {
        return "AB_xsA:" + s;
    }
    String xsABM(String s) {
        return "ABMxsA:" + s;
    }
}

class SPRB extends SPRA {
    String xsAB_(String s) {
        return "AB_xsB:" + s;
    }
    String xsABM(String s) {
        return "ABMxsB:" + s;
    }
    String xs_B_(String s) {
        return "_B_xsB:" + s;
    }
    String xs_BM(String s) {
        return "_BMxsB:" + s;
    }
}

public class MethodReferenceTestSuper extends SPRB {
    String xsA_M(String s) {
        return "A_MxsM:" + s;
    }
    String xsABM(String s) {
        return "ABMxsM:" + s;
    }
    String xs_BM(String s) {
        return "_BMxsM:" + s;
    }
    @Test
    public void testMethodReferenceSuper() {
        SPRI q = super::xsA__;
        assertEquals("A__xsA:*", q.m("*"));
        q = super::xsA_M;
        assertEquals("A_MxsA:*", q.m("*"));
        q = super::xsAB_;
        assertEquals("AB_xsB:*", q.m("*"));
        q = super::xsABM;
        assertEquals("ABMxsB:*", q.m("*"));
        q = super::xs_B_;
        assertEquals("_B_xsB:*", q.m("*"));
        q = super::xs_BM;
        assertEquals("_BMxsB:*", q.m("*"));
    }
}
