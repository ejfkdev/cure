import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestKinds extends MethodReferenceTestKindsSup {
    interface S0 {
        String get();
    }
    interface S1 {
        String get(MethodReferenceTestKinds x);
    }
    interface S2 {
        String get(MethodReferenceTestKinds x, MethodReferenceTestKinds y);
    }
    interface SXN0 {
        MethodReferenceTestKindsBase make(MethodReferenceTestKinds x);
    }
    interface SXN1 {
        MethodReferenceTestKindsBase make(MethodReferenceTestKinds x, String str);
    }
    interface SN0 {
        MethodReferenceTestKindsBase make();
    }
    interface SN1 {
        MethodReferenceTestKindsBase make(String x);
    }
    class In extends MethodReferenceTestKindsBase {
        In(String val) {
            this.val = val;
        }
        In() {
            this("blank");
        }
    }
    String instanceMethod0() {
        return "IM:0-" + this;
    }
    String instanceMethod1(MethodReferenceTestKinds x) {
        return "IM:1-" + this + x;
    }
    static String staticMethod0() {
        return "SM:0";
    }
    static String staticMethod1(MethodReferenceTestKinds x) {
        return "SM:1-" + x;
    }
    MethodReferenceTestKinds() {
        super("blank");
    }
    MethodReferenceTestKinds inst(String val) {
        var inst = new MethodReferenceTestKinds();
        inst.val = val;
        return inst;
    }
    @Test
    public void testMRBound() {
        S0 var = this::instanceMethod0;
        assertEquals("IM:0-MethodReferenceTestKinds(blank)", var.get());
    }
    @Test
    public void testMRBoundArg() {
        S1 var = this::instanceMethod1;
        assertEquals("IM:1-MethodReferenceTestKinds(blank)MethodReferenceTestKinds(arg)", var.get(inst("arg")));
    }
    @Test
    public void testMRUnbound() {
        S1 var = MethodReferenceTestKinds::instanceMethod0;
        assertEquals("IM:0-MethodReferenceTestKinds(rcvr)", var.get(inst("rcvr")));
    }
    @Test
    public void testMRUnboundArg() {
        S2 var = MethodReferenceTestKinds::instanceMethod1;
        assertEquals("IM:1-MethodReferenceTestKinds(rcvr)MethodReferenceTestKinds(arg)", var.get(inst("rcvr"), inst("arg")));
    }
    @Test
    public void testMRSuper() {
        S0 var = super::instanceMethod0;
        assertEquals("SIM:0-MethodReferenceTestKinds(blank)", var.get());
    }
    @Test
    public void testMRSuperArg() {
        S1 var = super::instanceMethod1;
        assertEquals("SIM:1-MethodReferenceTestKinds(blank)MethodReferenceTestKinds(arg)", var.get(inst("arg")));
    }
    @Test
    public void testMRStatic() {
        S0 var = MethodReferenceTestKinds::staticMethod0;
        assertEquals("SM:0", var.get());
    }
    @Test
    public void testMRStaticArg() {
        S1 var = MethodReferenceTestKinds::staticMethod1;
        assertEquals("SM:1-MethodReferenceTestKinds(arg)", var.get(inst("arg")));
    }
    @Test
    public void testMRTopLevel() {
        SN0 var = MethodReferenceTestKindsBase::new;
        assertEquals("MethodReferenceTestKindsBase(blank)", var.make().toString());
    }
    @Test
    public void testMRTopLevelArg() {
        SN1 var = MethodReferenceTestKindsBase::new;
        assertEquals("MethodReferenceTestKindsBase(name)", var.make("name").toString());
    }
    @Test
    public void testMRImplicitInner() {
        SN0 var = MethodReferenceTestKinds.In::new;
        assertEquals("In(blank)", var.make().toString());
    }
    @Test
    public void testMRImplicitInnerArg() {
        SN1 var = MethodReferenceTestKinds.In::new;
        assertEquals("In(name)", var.make("name").toString());
    }
}

class MethodReferenceTestKindsBase {
    String val = "unset";
    public String toString() {
        return getClass().getSimpleName() + "(" + val + ")";
    }
    MethodReferenceTestKindsBase(String val) {
        this.val = val;
    }
    MethodReferenceTestKindsBase() {
        this("blank");
    }
}

class MethodReferenceTestKindsSup extends MethodReferenceTestKindsBase {
    String instanceMethod0() {
        return "SIM:0-" + this;
    }
    String instanceMethod1(MethodReferenceTestKinds x) {
        return "SIM:1-" + this + x;
    }
    MethodReferenceTestKindsSup(String val) {
        super(val);
    }
}
