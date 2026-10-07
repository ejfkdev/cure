import java.util.*;

public class T8326204a {
    void testOneParam() {
        Object value = new ArrayList<String>();
        Object returnedValue = switch (1) {
            default -> {
                yield (List<String>) value;
            }
        };
    }
    void testTwoParams() {
        Object value = new HashMap<String, String>();
        Object returnedValue = switch (1) {
            default -> {
                yield (Map<String, String>) value;
            }
        };
    }
    void testTwoParamsInParens() {
        Object value = new HashMap<String, String>();
        Object returnedValue = switch (1) {
            default -> {
                yield ((Map<String, String>) value);
            }
        };
    }
}
