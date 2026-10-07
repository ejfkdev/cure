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

public class MethodRefToInner extends MethodRefToInnerBase {
    public static void main(String[] args) {
        new MethodRefToInner().run();
    }
    MethodRefToInner() {
        super();
    }
    void run() {
        new ArrayList<>().stream().forEach(TestString::new);
    }
}
