import java.util.*;

class T8016177e {
    interface TerminalOp<X, Y> {
    }
    interface Consumer<X> {
        void m(X x);
    }
    <T> TerminalOp<T, Void> makeRef(Consumer<? super T> action) {
        return null;
    }
    <T> void test() {
        TerminalOp<T, Void> forEachOp = makeRef((t) -> {
            null.put(t, null);
        });
    }
}
