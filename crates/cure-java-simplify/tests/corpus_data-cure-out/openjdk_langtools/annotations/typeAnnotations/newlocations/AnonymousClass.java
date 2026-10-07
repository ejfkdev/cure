import java.lang.annotation.*;

class AnonymousClass {
    Object o1;
    () { };
    Object o2;
    () { };
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TA {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TB {
}
