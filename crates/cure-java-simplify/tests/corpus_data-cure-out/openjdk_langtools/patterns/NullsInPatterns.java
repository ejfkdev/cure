import java.util.List;

public class NullsInPatterns {
    public static void main(String... args) {
        if (null instanceof List t) {
            throw new AssertionError("broken");
        } else {
            System.out.println("null does not match List type pattern");
        }
        if (null instanceof List<?> l) {
            throw new AssertionError("broken");
        } else {
            System.out.println("null does not match List<?> type pattern");
        }
    }
}
