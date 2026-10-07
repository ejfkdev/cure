public class PrimitiveInstanceOfConstantLocalVerifyError {
    void run() {
        if (!(42 instanceof byte b) || b != 42) {
            throw new AssertionError("primitive pattern failed");
        }
        if (!("hi" instanceof CharSequence cs) || !cs.toString().equals("hi")) {
            throw new AssertionError("reference pattern failed");
        }
    }
    public static void main(String[] args) {
        new PrimitiveInstanceOfConstantLocalVerifyError().run();
    }
}
