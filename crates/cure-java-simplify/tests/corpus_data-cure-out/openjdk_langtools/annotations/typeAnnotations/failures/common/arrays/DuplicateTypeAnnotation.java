import java.lang.annotation.*;

class DuplicateTypeAnnotation {
    void test() {}
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}
