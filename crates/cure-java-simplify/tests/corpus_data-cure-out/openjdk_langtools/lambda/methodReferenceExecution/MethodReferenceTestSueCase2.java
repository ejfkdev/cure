import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestSueCase2 {
    public interface Sam2<T> {
        public String get(T target, String s);
    }
    String instanceMethod(String s) {
        return "2";
    }
    static Sam2<MethodReferenceTestSueCase2> var = MethodReferenceTestSueCase2::instanceMethod;
    String m() {
        return var.get(new MethodReferenceTestSueCase2(), "");
    }
    @Test
    public void testSueCase2() {
        assertEquals("2", m());
    }
}
