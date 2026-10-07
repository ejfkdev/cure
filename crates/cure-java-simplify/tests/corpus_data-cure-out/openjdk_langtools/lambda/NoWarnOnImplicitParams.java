import java.util.List;

class NoWarnOnImplicitParams {
    public void testRawMerge(List<String> ls) {}
    interface R1 {
        Object m(List<String> ls);
    }
    @SuppressWarnings("rawtypes") interface R2 {
        String m(List l);
    }
    interface R12 extends R1, R2 {
    }
}
