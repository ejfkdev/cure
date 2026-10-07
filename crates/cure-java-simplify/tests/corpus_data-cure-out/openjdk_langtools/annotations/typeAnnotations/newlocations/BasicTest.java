import java.lang.annotation.*;
import java.util.*;
import java.io.*;

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface C {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface D {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface E {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface F {
}

class BasicTest<@D T extends @A Object> extends @B LinkedList<@E T> implements @C List<@F T> {
    void test() {
        Object o = (Object) "foo";
        String s = new @A String("bar");
        boolean b = o instanceof Object;
        @A Map<@B List<@C String>, @D String> map =
                    new @A HashMap<@B List<@C String>, @D String>();
        Class<? extends @A String> c2 = null;
    }
    void test2(@C @D BasicTest<T> this) throws @A IllegalArgumentException, @B IOException {}
    void test3(@B Object... objs) {}
    void test4(@B Class<@C ?>... clz) {}
}
