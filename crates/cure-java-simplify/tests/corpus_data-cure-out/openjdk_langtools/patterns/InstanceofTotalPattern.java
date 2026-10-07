public class InstanceofTotalPattern {
    public static void main(String[] args) {
        new InstanceofTotalPattern().totalTest();
    }
    void totalTest() {
        if (!("" instanceof String s1)) {
            throw new AssertionError();
        }
        if (null instanceof String s2) {
            throw new AssertionError();
        }
    }
}
