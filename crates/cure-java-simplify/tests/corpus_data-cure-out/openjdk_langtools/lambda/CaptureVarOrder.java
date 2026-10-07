public class CaptureVarOrder {
    static Object m(String s, int i, Object o) {
        return new Object() {
            final byte B = 0;
            void g() { System.out.println(s + i + B + o); }
        };
    }
    static Runnable r(String s, int i, Object o) {
        return () -> System.out.println(s + i + 0 + o);
    }
    public static void main(String[] args) throws ReflectiveOperationException {
        CaptureVarOrder.class.getDeclaredMethod("lambda$r$0", String.class, int.class, Object.class);
        new Object() {
            final byte B = 0;
            void g() { System.out.println(s + i + B + o); }
        }.getClass().getDeclaredConstructor(String.class, int.class, Object.class);
    }
}
