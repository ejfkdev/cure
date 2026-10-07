import java.lang.annotation.*;

class OldArray {
    [@A]  s() { return null; }
}

@Target(ElementType.TYPE_USE) @interface A {
}
