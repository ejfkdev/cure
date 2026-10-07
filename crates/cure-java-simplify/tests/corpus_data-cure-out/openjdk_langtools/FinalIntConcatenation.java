public class FinalIntConcatenation {
    public static void main(String[] args) throws Exception {
        if ("7".indexOf("null", 0) != -1) {
            throw new Exception("incorrect compile-time evaluation of final int");
        }
    }
}
