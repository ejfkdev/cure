public class T8291657 {
    static class A {
    }
    interface B {
    }
    static void f(final B b) {}
    static public B minimized(Object o) {
        return (B) switch (o) {
            default -> new A();
        };
    }
    public static void main(final String... args) {
        f((B) switch (new Object()) {
            case Object obj -> new A() {};
        });
    }
}
