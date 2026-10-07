import java.lang.annotation.*;

class MissingAnnotationValue<K> {
    MissingAnnotationValue<@A ?> l;
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
