import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

interface IDSs {
    String m(String a);
}

interface InDefA {
    default String xsA__(String s) {
        return "A__xsA:" + s;
    }
    default String xsAB_(String s) {
        return "AB_xsA:" + s;
    }
}

interface InDefB extends InDefA {
    default String xsAB_(String s) {
        return "AB_xsB:" + s;
    }
    default String xs_B_(String s) {
        return "_B_xsB:" + s;
    }
}

public class MethodReferenceTestInnerDefault implements InDefB {
    @Test
    public void testMethodReferenceInnerDefault() {
        new In().testMethodReferenceInnerDefault();
    }
    class In {
        public void testMethodReferenceInnerDefault() {
            IDSs q = MethodReferenceTestInnerDefault.this::xsA__;
            assertEquals("A__xsA:*", q.m("*"));
            q = MethodReferenceTestInnerDefault.this::xsAB_;
            assertEquals("AB_xsB:*", q.m("*"));
            q = MethodReferenceTestInnerDefault.this::xs_B_;
            assertEquals("_B_xsB:*", q.m("*"));
        }
    }
}
