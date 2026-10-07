public class EmptyRecordClass {
    record X() {
    }
    void test(X w) {
        switch (w) {
            case X():
                break;
        }
    }
    sealed interface W permits W.X1 {
        record X1() implements W {
        }
    }
    public int test2(W w) {
        return switch (w) {
            case W.X1() -> 1;
        };
    }
}
