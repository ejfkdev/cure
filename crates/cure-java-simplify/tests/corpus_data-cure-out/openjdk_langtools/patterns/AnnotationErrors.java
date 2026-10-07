import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class AnnotationErrors {
    private void test(Object o, G<String> g) {
        boolean b1 = o instanceof R(var s);
        boolean b2 = o instanceof R(var s);
        boolean b3 = o instanceof R(var s);
        boolean b5 = g instanceof G<@DTA String>(var s);
        boolean b6 = g instanceof G<@TA String>(var s);
        switch (o) {
            case R(var s) when b1 -> {}
            case R(var s) when b1 -> {}
            case R(var s) when b1 -> {}
            default -> {}
        }
        switch (g) {
            case G<@DTA String>(var s) when b1 -> {}
            case G<@TA String>(var s) when b1 -> {}
            default -> {}
        }
    }
    record R(String s) {
    }
    record G<T>(T t) {
    }
    @Target(ElementType.LOCAL_VARIABLE) @interface DA {
    }
    @Target({ElementType.TYPE_USE, ElementType.LOCAL_VARIABLE}) @interface DTA {
    }
    @Target(ElementType.TYPE_USE) @interface TA {
    }
}
