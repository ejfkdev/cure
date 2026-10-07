public class T8309054 {
    public void test(Object obj) {
        boolean t1 = switch (obj) {
            case Long a[] -> true;
            default -> false;
        };
        boolean t2 = switch (obj) {
            case Double a[][][][] -> true;
            default -> false;
        };
        if (obj instanceof Integer a = Integer.valueOf(0)) {}
    }
}
