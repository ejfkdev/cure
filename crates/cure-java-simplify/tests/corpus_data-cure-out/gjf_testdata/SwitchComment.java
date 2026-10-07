class T {
    void f(String v) {}
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
