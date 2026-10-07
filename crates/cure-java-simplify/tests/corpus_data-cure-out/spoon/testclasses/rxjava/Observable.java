package spoon.test.generics.testclasses.rxjava;

import java.util.Objects;

public class Observable<T> implements Publisher<T> {
    public final void subscribe(Subscriber<? super T> s) {
        Objects.requireNonNull(s);
    }
}
