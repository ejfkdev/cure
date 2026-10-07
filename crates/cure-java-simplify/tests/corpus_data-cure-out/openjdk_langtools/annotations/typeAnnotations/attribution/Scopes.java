import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

class Scopes {
    void test(@A(VALUE) Scopes this) {}
    void test1(@A(value=VALUE) Scopes this) {}
    private static final int VALUE = 1;
    @Target(ElementType.TYPE_USE) @interface A {
        int value();
    }
}
