import java.lang.annotation.*;
import java.util.List;

class AnnotatedClassExpr {
    static void main() {
        Object o1 = int[].class;
        o1 = int @A [] .
        class;
        o1 = int[][].class;
        o1 = AnnotatedClassExpr @A [] .
        class;
        o1 = @A AnnotatedClassExpr @A [] .
        class;
        o1 = AnnotatedClassExpr.class;
    }
}

@Target(ElementType.TYPE_USE) @interface A {
}
