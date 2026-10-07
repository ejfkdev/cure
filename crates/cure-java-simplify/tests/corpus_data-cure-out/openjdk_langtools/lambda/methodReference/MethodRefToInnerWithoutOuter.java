import java.util.List;
import java.util.ArrayList;

class MethodRefToInnerBase {
    class TestString {
        String str;
        TestString(String strin) {
            str = strin;
        }
    }
}

public class MethodRefToInnerWithoutOuter extends MethodRefToInnerBase {
    public static void meth() {
        new ArrayList<>().stream().forEach(TestString::new);
    }
}
