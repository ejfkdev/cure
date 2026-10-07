import java.lang.annotation.Target;
import java.lang.annotation.ElementType;

public class T8312560 {
    void m(Object o) {}
    @interface A {
    }
    record R(Integer x) {
    }
}
