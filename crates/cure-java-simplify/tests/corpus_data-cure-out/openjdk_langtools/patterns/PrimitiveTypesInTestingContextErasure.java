import java.util.List;

public class PrimitiveTypesInTestingContextErasure {
    public static void main(String[] args) {
        erasureSwitch();
        erasureInstanceofTypeComparisonOperator();
        erasureInstanceofPatternMatchingOperator();
        pollutedInstanceofPatternMatchingOperatorReference();
        pollutedInstanceofPatternMatchingOperator();
        pollutedInstanceofTypeComparisonOperator();
        pollutedSwitch();
    }
    public static void erasureSwitch() {
        List<Short> ls = List.of((short) 42);
        Short s = 42;
        assertTrue(switch (ls.get(0)) {
            case int _ -> true;
            default -> false;
        });
    }
    public static void erasureInstanceofTypeComparisonOperator() {
        assertTrue(List.of((short) 42).get(0) instanceof int);
    }
    public static void erasureInstanceofPatternMatchingOperator() {
        assertTrue(List.of((short) 42).get(0) instanceof int i);
    }
    public static void pollutedInstanceofPatternMatchingOperator() {
        assertTrue(!(((List) List.of("42")).get(0) instanceof int i));
    }
    public static void pollutedInstanceofTypeComparisonOperator() {
        assertTrue(!(((List) List.of("42")).get(0) instanceof int));
    }
    public static void pollutedInstanceofPatternMatchingOperatorReference() {
        assertTrue(!(((List) List.of("42")).get(0) instanceof Short));
    }
    public static void pollutedSwitch() {
        List<Short> ls = (List) List.of("42");
        try {
            var res = switch (ls.get(0)) {
                case int _ -> true;
                default -> false;
            };
            throw new AssertionError("Expected: ClassCastException");
        } catch (ClassCastException e) {}
    }
    static void assertTrue(boolean actual) {
        if (!actual) {
            throw new AssertionError("Expected: true, but got false");
        }
    }
}
