import java.lang.annotation.*;

class IncompleteArray {
    int[] var;
}

@Target(ElementType.TYPE_USE) @interface A {
}
