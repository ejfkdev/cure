import java.util.Objects;

public class NestedPrimitiveDeconstructionPattern {
    public static void main(String... args) throws Throwable {
        new NestedPrimitiveDeconstructionPattern().doTestR();
    }
    void doTestR() {
        assertEquals("OK", switchR1(new R(3, 42d)));
        assertEquals("OK", switchR1_int_double(new R_i(3, 42d)));
    }
    record R(Integer x, Double y) {
    }
    String switchR1(R r) {
        return switch (r) {
            case R(Integer x, Double y) -> "OK";
        };
    }
    record R_i(int x, double y) {
    }
    String switchR1_int_double(R_i r) {
        return switch (r) {
            case R_i(int x, double y) -> "OK";
        };
    }
    private void assertEquals(String expected, String actual) {
        if (!Objects.equals(expected, actual)) {
            throw new AssertionError("Expected: " + expected + ", but got: " + actual);
        }
    }
}
