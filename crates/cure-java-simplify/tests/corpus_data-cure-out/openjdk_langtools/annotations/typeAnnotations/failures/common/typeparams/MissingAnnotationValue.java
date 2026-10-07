import java.lang.annotation.*;

class MissingAnnotationValue<@A K> {
}

@Target(ElementType.TYPE_USE) @interface A {
    int field();
}
