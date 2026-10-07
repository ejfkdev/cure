import java.lang.annotation.*;

class DuplicateAnnotationValue {
    void test() {
        String[] s;
    }
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
    int value();
}
