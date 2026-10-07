public class NoModifiersOnBinding {
    private static void test(Object o) {
        if (o instanceof String) {
            System.err.println(s);
        }
        if (o instanceof String) {
            System.err.println(s);
        }
        if (o instanceof String s) {
            System.err.println(s);
        }
        if (o instanceof String s) {
            System.err.println(s);
        }
    }
}
