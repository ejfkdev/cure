package spoon.test.generics.testclasses;

import java.util.Iterator;
import java.util.function.Consumer;

public class SameSignature<T extends String> implements Iterable<T> {
    @Override
    public Iterator<T> iterator() {
        return null;
    }
    @Override
    public void forEach(Consumer<? super T> action) {}
}
