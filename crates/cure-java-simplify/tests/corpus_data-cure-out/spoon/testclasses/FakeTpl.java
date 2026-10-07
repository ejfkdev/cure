package spoon.test.generics.testclasses;

import spoon.reflect.declaration.CtElement;
import spoon.reflect.declaration.CtNamedElement;
import spoon.reflect.declaration.CtType;

public interface FakeTpl<T extends CtElement> {
    T apply(CtType<? extends CtNamedElement> targetType);
    String test(CtType<?> something, int i, T bidule);
}
