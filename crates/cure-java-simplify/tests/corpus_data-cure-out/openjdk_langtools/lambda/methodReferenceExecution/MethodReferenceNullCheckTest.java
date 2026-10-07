import java.util.function.Supplier;

public class MethodReferenceNullCheckTest {
    public static void main(String[] args) {
        boolean npeFired = false;
        try {
            Supplier<Boolean> ss = null::isEmpty;
        } catch (NullPointerException npe) {
            npeFired = true;
        } finally {
            if (!npeFired) 
                throw new AssertionError("NPE should have been thrown");
        }
    }
}
