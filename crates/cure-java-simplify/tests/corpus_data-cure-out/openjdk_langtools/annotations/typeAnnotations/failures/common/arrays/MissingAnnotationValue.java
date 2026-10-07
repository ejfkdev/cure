import java.lang.annotation.*;

class MissingAnnotationValue {
    void test() {
        String[] s;
    }
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
