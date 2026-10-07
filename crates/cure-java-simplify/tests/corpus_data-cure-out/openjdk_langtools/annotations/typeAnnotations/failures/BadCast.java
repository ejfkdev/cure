import java.lang.annotation.*;

class BadCast {
    static void main() {}
}

@Target(ElementType.TYPE_USE) @interface A {
}
