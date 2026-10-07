public class EnsureTypesOrderTest {
    public static void meth(String[] args) {
        if (args instanceof String s) {
            System.out.println("Broken");
        }
    }
}
