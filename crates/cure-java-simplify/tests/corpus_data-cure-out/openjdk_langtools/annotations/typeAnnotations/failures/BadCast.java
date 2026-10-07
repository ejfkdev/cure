import java.lang.annotation.*;

class BadCast {
    static void main() {
        Object o = "";
    }
}

@Target(ElementType.TYPE_USE) @interface A {
}
