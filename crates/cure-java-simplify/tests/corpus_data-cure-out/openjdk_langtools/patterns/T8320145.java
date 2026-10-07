public class T8320145 {
    record ARecord(String aComponent) {
    }
    record BRecord(ARecord aComponent) {
    }
    record CRecord(ARecord aComponent1, ARecord aComponent2) {
    }
    public String match(Object o) {
        return switch (o) {
            case ARecord(final String s) -> s;
            case BRecord(ARecord(final String s)) -> s;
            case CRecord(ARecord(String s), ARecord(final String s2)) -> s;
            default -> "No match";
        };
    }
}
