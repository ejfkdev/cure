import java.lang.annotation.*;

class DuplicateAnnotationValue {
    void test() {}
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
    int value();
}
