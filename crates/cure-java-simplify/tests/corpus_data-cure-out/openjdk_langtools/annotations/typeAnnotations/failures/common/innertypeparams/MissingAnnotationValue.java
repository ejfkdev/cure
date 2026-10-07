import java.lang.annotation.*;

class MissingAnnotationValue {
    void innermethod() {
        class Inner<@A K> { }
    }
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
