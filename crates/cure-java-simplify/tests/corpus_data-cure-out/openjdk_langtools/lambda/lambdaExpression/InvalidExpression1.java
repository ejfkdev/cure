import java.util.Comparator;

public class InvalidExpression1 {
    void test() {
        Comparator<Number> c = (Number n1, Number n2) -> {
            42;
        };
        Comparator<Number> c = (Number n1, Number n2) -> {
            return 42;
        };
    }
}
