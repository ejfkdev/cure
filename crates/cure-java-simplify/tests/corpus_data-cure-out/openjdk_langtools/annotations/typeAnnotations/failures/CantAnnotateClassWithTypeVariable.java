import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class CantAnnotateClassWithTypeVariable {
    @Target(ElementType.TYPE_USE) @interface TA {
    }
    static class A {
        static class B<T> {
        }
    }
    <T> A.B<T> f() {}
}
