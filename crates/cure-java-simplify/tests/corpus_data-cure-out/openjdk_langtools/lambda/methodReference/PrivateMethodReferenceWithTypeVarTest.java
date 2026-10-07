import java.util.function.*;

class PrivateMethodReferenceWithTypeVarTest {
    class Foo<X> {
        X get() {
            return null;
        }
    }
    private String asString() {
        return "bar";
    }
    private String asString2(Object o) {
        return "bar";
    }
    static <T extends PrivateMethodReferenceWithTypeVarTest> Function<T, String> m1() {
        return T::asString;
    }
    static <T extends PrivateMethodReferenceWithTypeVarTest> Function<T, String> m2(T t) {
        return t::asString2;
    }
    static Function<?, String> m2(Foo<? extends PrivateMethodReferenceWithTypeVarTest> foo) {
        return foo.get()::asString2;
    }
}
