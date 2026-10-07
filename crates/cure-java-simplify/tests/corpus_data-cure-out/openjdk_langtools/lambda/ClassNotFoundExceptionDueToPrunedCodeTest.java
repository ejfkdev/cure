public class ClassNotFoundExceptionDueToPrunedCodeTest {
    public static void main(String... args) {
        var o1 = null;
        Runnable r = () -> {
            System.out.println(o1 == o1);
        };
        r.run();
        var o2 = null;
        r = () -> {
            System.out.println(o2 == o2);
        };
        r.run();
        var o3 = switch (0) {
            default -> {
                yield null;
            }
        };
        r = () -> System.out.println(o3);
        r.run();
        var o4 = switch (0) {
            default -> {
                yield null;
            }
        };
        r = () -> System.out.println(o4);
        r.run();
    }
}
