import java.util.ArrayList;

public class UncheckedWarningOnMatchesTest {
    public static void meth() {
        Object o = new ArrayList<UncheckedWarningOnMatchesTest>();
        if (o instanceof ArrayList<Integer> ai) {
            System.out.println("Blah");
        }
    }
}
