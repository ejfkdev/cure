public class ImpossibleTypeTest {
    public static void meth() {
        Integer i = 42;
        if (i instanceof String s) {
            System.out.println("Broken");
        }
        if (i instanceof Undefined u) {
            System.out.println("Broken");
        }
    }
}
