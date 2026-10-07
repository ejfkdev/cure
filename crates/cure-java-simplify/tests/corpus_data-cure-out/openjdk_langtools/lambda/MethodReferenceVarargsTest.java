public class MethodReferenceVarargsTest<T> {
    public T invoke(Object... args) {
        return null;
    }
    public static <T extends String> void test() {
        java.util.function.Function<String, T> f = (args) -> new MethodReferenceVarargsTest<>().invoke(args);
    }
}
