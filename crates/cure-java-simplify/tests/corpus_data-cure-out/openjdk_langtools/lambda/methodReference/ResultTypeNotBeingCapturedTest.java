import java.util.function.Supplier;

class ResultTypeNotBeingCapturedTest {
    interface X<T> {
        X<T> self();
    }
    static X<?> makeX() {
        return null;
    }
    static <R> X<R> create(Supplier<? extends R> supplier) {
        return null;
    }
    static X<X<?>> methodRef() {
        return create(ResultTypeNotBeingCapturedTest::makeX).self();
    }
    static X<X<?>> lambda() {
        return create(() -> makeX()).self();
    }
}
