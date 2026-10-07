import java.util.*;

public class MethodReference11 {
    public static void main(String[] args) {
        String[] strings = {"D", "C", "B", "A"};
        Arrays.sort(strings, String.CASE_INSENSITIVE_ORDER::compare);
        for (String s : strings) {
            if (String.CASE_INSENSITIVE_ORDER.compare("1", s) > 0) {
                throw new AssertionError();
            }
        }
    }
}
