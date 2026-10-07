package com.puppycrawl.tools.checkstyle.checks.whitespace.genericwhitespace;

import java.util.Collections;
import java.util.List;
import java.util.function.Supplier;

public class InputGenericWhitespaceWithEmoji {
    public static class SomeClass {
        public static class Nested<V> {
            private Nested() {}
        }
    }
    public <V> void methodName(V value) {
        Supplier<?> t = InputGenericWhitespaceMethodRef1.Nested2::new;
    }
    interface NumberEnum<T
 > {
    }
    public int getConstructor(Class<?>... parameterTypes) {
        Collections.emptySet();
        Collections.emptySet();
        return 666;
    }
    Object ok2;
    ();
    Object notOkStart;
    (); // violation ''<' is not preceded with whitespace'
        /* 😆asd*/
    public static class IntEnumValueType3<E extends Enum<E>> {
    }
}
