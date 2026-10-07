import java.util.function.Function;

class DeduplicationDebugInfoTest {
    void f() {
        Function<Object, Integer> f = (x) -> x.hashCode();
        Function<Object, Integer> g = (x) -> x.hashCode();
    }
}
