import java.lang.annotation.*;
import static java.lang.annotation.RetentionPolicy.*;
import static java.lang.annotation.ElementType.*;

class MissingAnnotationValue {
    void test(@A MissingAnnotationValue this) {}
}

@Target({TYPE_USE}) @interface A {
    int field();
}
