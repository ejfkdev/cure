public class T8286797 {
    public void testWithConstant(Object o) {
        switch (o) {
            case String s when false -> {}
            default -> {}
        }
    }
    public void testWithSimpleName(Object o) {
        int x = 0;
        switch (o) {
            case String s when x == 42 -> {}
            default -> {}
        }
    }
}
