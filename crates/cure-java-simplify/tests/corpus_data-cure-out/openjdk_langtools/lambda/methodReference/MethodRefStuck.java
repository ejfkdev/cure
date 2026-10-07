import java.util.Optional;
import java.util.stream.Stream;

public abstract class MethodRefStuck {
    public static void main(Stream<String> xs, Optional<String> x) {
        xs.map((c) -> {
            return new I(x.map(c::equals));
        });
    }
    static class I {
        I(boolean i) {}
    }
}
