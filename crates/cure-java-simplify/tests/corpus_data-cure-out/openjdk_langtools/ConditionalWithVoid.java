public class ConditionalWithVoid {
    public void test(Object o, String s) {
        System.out.println(o.hashCode());
        o.hashCode().toString();
        System.out.println(switch (s) {
            case "" -> o.hashCode();
            default -> o.wait();
        });
        (switch (s) {
            case "" -> o.hashCode();
            default -> o.wait();
        }).toString();
    }
}
