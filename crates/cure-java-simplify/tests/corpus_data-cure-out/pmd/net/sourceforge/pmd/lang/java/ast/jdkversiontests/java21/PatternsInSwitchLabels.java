public class PatternsInSwitchLabels {
    public static void main(String[] args) {
        Object o = 123L;
        String formatted = switch (o) {
            case Integer i -> String.format("int %d", i);
            case Long l -> String.format("long %d", l);
            case Double d -> String.format("double %f", d);
            case String s -> String.format("String %s", s);
            default -> o.toString();
        };
        System.out.println(formatted);
    }
}
