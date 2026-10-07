import java.lang.annotation.*;

class MissingAnnotationValue {
    void test() {
        String[] a = new String @A [5];
    }
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
