import java.util.function.Supplier;

public class MethodReferenceInConstructorInvocation {
    interface Bar {
        default String getString() {
            return "";
        }
    }
    static class Foo implements Bar {
        Foo() {
            this(Bar.super::getString);
        }
        Foo(Supplier<String> sString) {}
        Foo(int i) {
            this(Bar.super.getString());
        }
        Foo(String s) {}
    }
}
