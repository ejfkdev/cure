public class RawTypeBindingWarning<T> {
    public static boolean t(Object o) {
        return o instanceof RawTypeBindingWarning w;
    }
    public static void t2(Object o) {
        switch (o) {
            case RawTypeBindingWarning w -> {}
            default -> {}
        }
        switch (o) {
            case RawTypeBindingWarning w when w == null -> {}
            default -> {}
        }
    }
}
