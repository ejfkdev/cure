public class T8336781 {
    public static void test() {
        var _ = switch (null) {
            case null -> "nothing";
            case true -> "something true";
            case false -> "something false";
        };
    }
}
