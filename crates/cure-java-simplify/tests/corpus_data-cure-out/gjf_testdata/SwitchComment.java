class T {
    void f(String v) {
        int x = switch (v) {
            case "zero" -> 0;
            case "one" -> 1;
            case "two" -> 2;
            default -> -1;
        };
    }
    void g(String v) {
        int x = switch (v) {
            case "zero":
                return 0;
            case "one":
                return 1;
            case "two":
                return 2;
            default:
                return -1;
        };
    }
}
