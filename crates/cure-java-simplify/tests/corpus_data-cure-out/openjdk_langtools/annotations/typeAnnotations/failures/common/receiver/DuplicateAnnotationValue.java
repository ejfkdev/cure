import java.lang.annotation.*;

class DuplicateAnnotationValue {
    void test(@A(value = 2, value = 1) DuplicateAnnotationValue this) {}
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
    int value();
}
