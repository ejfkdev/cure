import java.lang.annotation.*;

class MissingAnnotationValue {
    void test() {}
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
