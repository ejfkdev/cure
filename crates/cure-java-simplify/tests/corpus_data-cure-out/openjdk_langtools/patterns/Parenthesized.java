public class Parenthesized {
    public static void main(String... args) {
        new Parenthesized().run();
    }
    void run() {
        Object o = "";
        switch (o) {
            case (String s) when s.isEmpty() -> System.err.println("OK: " + s);
            default -> throw new AssertionError();
        }
        System.err.println(switch (o) {
            case (String s) when s.isEmpty() -> "OK: " + s;
            default -> throw new AssertionError();
        });
        if (o instanceof (String s) && s.isEmpty()) {
            System.err.println("OK: " + s);
        }
        boolean b1 = o instanceof (String s) && s.isEmpty();
        boolean b2 = o instanceof String s && s.isEmpty();
    }
}
