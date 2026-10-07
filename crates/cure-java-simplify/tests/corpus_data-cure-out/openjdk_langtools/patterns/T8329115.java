public class T8329115 {
    record R1() {
    }
    record R2() {
    }
    int test() {
        return switch (new R1()) {
            case R1() -> {
                return switch (new R2()) {
                    case R2() -> 1;
                };
            }
        };
    }
}
